use std::path::Path;

use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::{AdamW, Optimizer, VarMap};
use candle_transformers::models::llama::{self, Llama};
use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;

use crate::model;
use crate::spec::{Magnitude, MemoryAction, ReflectionAction};
use crate::tokenizer::TokenizerWrapper;

// ── Data formatting ──────────────────────────────────────────────────────

fn format_input(example: &crate::dataset::Example) -> String {
    let input = &example.input;
    let mut lines = Vec::new();

    let traits: Vec<&str> = input.character.personality.iter().map(|s| s.as_str()).collect();
    lines.push(format!("Personality: {}", traits.join(", ")));

    for (target, traits_map) in &input.relationship {
        for (trait_name, value) in traits_map {
            lines.push(format!("Relationship ({target}): {trait_name}={value}"));
        }
    }

    for (emotion, intensity) in &input.current_state {
        lines.push(format!("Current state: {emotion}={intensity}"));
    }

    if let Some(prev) = &input.previous_character_message {
        lines.push(format!("Previous: \"{prev}\""));
    }

    lines.push(format!("User: \"{}\"", input.user_message));
    lines.push("---".to_string());
    lines.join("\n")
}

fn format_target(changes: &crate::spec::StateChanges) -> String {
    match changes {
        crate::spec::StateChanges::NoChanges(_) => return "no_changes".to_string(),
        crate::spec::StateChanges::Record(inner) => {
            let mut lines = Vec::new();

            if let Some(emotions) = &inner.emotion {
                for (name, mag) in emotions {
                    lines.push(format!("emotion|{name}|{}", mag_to_str(*mag)));
                }
            }

            if let Some(traits) = &inner.relationship {
                for (trait_name, mag) in traits {
                    lines.push(format!("relationship|{trait_name}|{}", mag_to_str(*mag)));
                }
            }

            if let Some(beliefs) = &inner.belief {
                for (id, mag) in beliefs {
                    lines.push(format!("belief|{id}|{}", mag_to_str(*mag)));
                }
            }

            if let Some(memory) = &inner.memory {
                match memory {
                    MemoryAction::ReinforcePreviousConflict => {
                        lines.push("memory|reinforce_previous_conflict".to_string());
                    }
                }
            }

            if let Some(reflection) = &inner.reflection {
                match reflection {
                    ReflectionAction::Required => {
                        lines.push("reflection|required".to_string());
                    }
                }
            }

            lines.join("\n")
        }
    }
}

fn mag_to_str(m: Magnitude) -> &'static str {
    match m {
        Magnitude::IncreasesALot => "increases_a_lot",
        Magnitude::Increases => "increases",
        Magnitude::IncreasesALittle => "increases_a_little",
        Magnitude::DecreasesALittle => "decreases_a_little",
        Magnitude::Decreases => "decreases",
        Magnitude::DecreasesALot => "decreases_a_lot",
    }
}

// ── Training dataset ──────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TrainingExample {
    pub input_text: String,
    pub target_text: String,
}

pub fn load_training_data(data_dir: &Path) -> anyhow::Result<Vec<TrainingExample>> {
    let mut pairs = Vec::new();
    crate::dataset::collect_pairs(data_dir, &mut pairs);

    let mut examples = Vec::new();
    for dir in &pairs {
        let input_path = dir.join("input.yaml");
        let output_path = dir.join("output.yaml");
        match crate::dataset::load_example(&input_path, &output_path) {
            Ok(example) => {
                let input_text = format_input(&example);
                let target_text = format_target(&example.target.state_changes);
                examples.push(TrainingExample { input_text, target_text });
            }
            Err(e) => {
                eprintln!("warning: skipping {}: {e}", dir.display());
            }
        }
    }

    Ok(examples)
}

// ── Prepare tensors ──────────────────────────────────────────────────────

pub fn prepare_batch(
    examples: &[TrainingExample],
    tokenizer: &TokenizerWrapper,
    device: &Device,
) -> anyhow::Result<(Tensor, Vec<usize>, Vec<usize>)> {
    let mut batch_ids: Vec<Vec<u32>> = Vec::new();
    let mut sep_positions: Vec<usize> = Vec::new();
    let mut actual_lengths: Vec<usize> = Vec::new();

    for ex in examples {
        let input_ids = tokenizer.encode(&ex.input_text, false);
        let target_ids = tokenizer.encode(&ex.target_text, false);
        let sep_pos = input_ids.len();

        let mut combined = input_ids;
        combined.push(tokenizer.bos_id); // separator between input and target
        combined.extend(target_ids);
        combined.push(tokenizer.eos_id);

        actual_lengths.push(combined.len());
        sep_positions.push(sep_pos);
        batch_ids.push(combined);
    }

    // pad to max length
    let max_len = batch_ids.iter().map(|ids| ids.len()).max().unwrap_or(0);
    let mut padded: Vec<Vec<u32>> = Vec::new();
    for ids in &batch_ids {
        let mut row = ids.clone();
        row.resize(max_len, tokenizer.eos_id); // pad with EOS
        padded.push(row);
    }

    let shape = (padded.len(), max_len);
    let flat: Vec<u32> = padded.into_iter().flatten().collect();
    let tensor = Tensor::from_vec(flat, shape, device)?;
    Ok((tensor, sep_positions, actual_lengths))
}

fn loss_for_batch(
    model: &Llama,
    varmap: &VarMap,
    input_ids: &Tensor,
    sep_positions: &[usize],
    actual_lengths: &[usize],
    config: &llama::Config,
    device: &Device,
    debug: bool,
) -> anyhow::Result<Tensor> {
    let b = input_ids.dim(0)?;
    let mut cache = model::create_cache(false, DType::F32, config, device)?;
    let logits = model.forward_train(input_ids, &mut cache)?;
    let (_, logit_len, vocab_size) = logits.dims3()?;

    // targets = next token at each position (shifted by 1)
    let targets = input_ids.narrow(1, 1, logit_len)?;

    // Per-element cross-entropy
    let flat_logits = logits.reshape(((), vocab_size))?;
    let flat_targets = targets.flatten_all()?;
    let log_probs = candle_nn::ops::log_softmax(&flat_logits, 1)?;
    let gathered = log_probs.gather(&flat_targets.unsqueeze(1)?, 1)?.squeeze(1)?;
    let losses = gathered.neg()?.reshape((b, logit_len))?;

    // Build mask: 1 for target positions (from separator to actual end),
    // 0 for input positions and padding.
    // logit_len = max_padded_len - 1 (forward_train drops last position).
    // Each example's target region is [sep .. actual_len-1) in logit space.
    let mut mask = vec![0u8; b * logit_len];
    let mut supervised_count = 0usize;
    for bi in 0..b {
        let sep = sep_positions[bi];
        let end = actual_lengths[bi].saturating_sub(1); // last logit that has a real target
        for ti in sep..end {
            mask[bi * logit_len + ti] = 1;
            supervised_count += 1;
        }
    }
    let mask_t = Tensor::from_slice(&mask, (b, logit_len), device)?.to_dtype(DType::F32)?;

    if debug {
        crate::model::dump_weight_stats(varmap);

        let total_positions = b * logit_len;
        eprintln!(
            "  debug: b={b} logit_len={logit_len} total_positions={total_positions} supervised={supervised_count}"
        );

        // sample first example's logits for the first supervised position
        if b > 0 && supervised_count > 0 {
            let first_sep = sep_positions[0];
            let first_end = actual_lengths[0].saturating_sub(1);
            if first_sep < first_end {
                let sample_logits = logits.i((0, first_sep as usize, ..))?;
                let top_vals = candle_nn::ops::softmax(&sample_logits, 0)?;
                let mut top5: Vec<(f32, u32)> = top_vals
                    .to_vec1::<f32>()?
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (v, i as u32))
                    .collect();
                top5.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                let target_id = targets.to_vec2::<u32>()?[0][first_sep];
                eprintln!(
                    "  debug: first sep pos={first_sep}, target token ID={target_id}"
                );
                eprintln!("  debug: top-5 predictions at that position:");
                for (prob, id) in top5.iter().take(5) {
                    eprintln!("    token {id:>5}: {prob:.6}");
                }
            }
        }
    }

    let masked = (losses * mask_t.clone())?;
    let sum_loss = masked.sum_all()?;
    let count = mask_t.sum_all()?;
    Ok((sum_loss / count)?)
}

// ── Training loop ─────────────────────────────────────────────────────────

pub struct TrainConfig {
    pub epochs: usize,
    pub learning_rate: f64,
    pub batch_size: usize,
    pub checkpoint_every: usize,
    pub output: std::path::PathBuf,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            epochs: 100,
            learning_rate: 0.001,
            batch_size: 0, // 0 = full batch
            checkpoint_every: 0,
            output: std::path::PathBuf::from("model.safetensors"),
        }
    }
}

pub fn train(
    model: &Llama,
    varmap: &VarMap,
    examples: &[TrainingExample],
    tokenizer: &TokenizerWrapper,
    device: &Device,
    config: &TrainConfig,
    llama_config: &llama::Config,
) -> anyhow::Result<()> {
    let vars = varmap.all_vars();
    let mut adam = AdamW::new_lr(vars.clone(), config.learning_rate)?;

    let batch_size = if config.batch_size == 0 {
        examples.len()
    } else {
        config.batch_size.min(examples.len())
    };

    let mut rng = rand::rng();
    let n_batches = (examples.len() + batch_size - 1) / batch_size;

    eprintln!(
        "training: {} examples across ~{} batches/epoch, {} epochs, lr={}",
        examples.len(),
        n_batches,
        config.epochs,
        config.learning_rate,
    );

    let pb = ProgressBar::new(config.epochs as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("=> "),
    );

    for epoch in 0..config.epochs {
        let mut indices: Vec<usize> = (0..examples.len()).collect();
        indices.shuffle(&mut rng);

        let mut epoch_loss = 0.0f64;
        let mut n_seen = 0;

        for batch_idx in 0..n_batches {
            let start = batch_idx * batch_size;
            let end = (start + batch_size).min(examples.len());
            let batch_examples: Vec<TrainingExample> =
                indices[start..end].iter().map(|&i| examples[i].clone()).collect();

            let (batch_tensor, sep_positions, actual_lengths) =
                prepare_batch(&batch_examples, tokenizer, device)?;
            let first_batch = epoch == 0 && batch_idx == 0;
            let loss = loss_for_batch(
                model,
                varmap,
                &batch_tensor,
                &sep_positions,
                &actual_lengths,
                llama_config,
                device,
                first_batch,
            )?;
            let loss_val = loss.to_scalar::<f32>()? as f64;

            adam.backward_step(&loss)?;

            epoch_loss += loss_val * batch_examples.len() as f64;
            n_seen += batch_examples.len();
        }

        let avg_loss = epoch_loss / n_seen as f64;
        pb.set_message(format!("epoch {:3}  loss {:.6}", epoch + 1, avg_loss));
        println!("Epoch: {:3} | loss {:.6}", epoch + 1, avg_loss);
        pb.inc(1);

        if config.checkpoint_every > 0 && (epoch + 1) % config.checkpoint_every == 0 {
            let stem = config.output.file_stem().unwrap_or_default().to_string_lossy();
            let checkpoint = format!("{}-epoch{}.safetensors", stem, epoch + 1);
            crate::model::save_model(varmap, Path::new(&checkpoint))?;
            eprintln!("\ncheckpoint saved: {checkpoint}");
        }
    }

    pb.finish_with_message("training complete");
    Ok(())
}
