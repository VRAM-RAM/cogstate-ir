use std::path::Path;

use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::{AdamW, Optimizer, VarMap};
use candle_transformers::models::llama::{self, Llama};
use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::model;
use crate::spec::{MemoryAction, ReflectionAction};
use crate::tokenizer::TokenizerWrapper;

// ── Data formatting ──────────────────────────────────────────────────────

pub fn format_input(input: &crate::spec::Input) -> String {
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
                    lines.push(format!("emotion|{name}|{}", mag.as_str()));
                }
            }

            if let Some(traits) = &inner.relationship {
                for (trait_name, mag) in traits {
                    lines.push(format!("relationship|{trait_name}|{}", mag.as_str()));
                }
            }

            if let Some(beliefs) = &inner.belief {
                for (id, mag) in beliefs {
                    lines.push(format!("belief|{id}|{}", mag.as_str()));
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

// ── Training dataset ──────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TrainingExample {
    pub input_ids: Vec<u32>,
    pub target_ids: Vec<u32>,
}

pub fn load_training_data(
    data_dir: &Path,
    tokenizer: &TokenizerWrapper,
) -> anyhow::Result<Vec<TrainingExample>> {
    let mut pairs = Vec::new();
    crate::dataset::collect_pairs(data_dir, &mut pairs);

    let mut examples = Vec::new();
    for dir in &pairs {
        let input_path = dir.join("input.yaml");
        let output_path = dir.join("output.yaml");
        match crate::dataset::load_example(&input_path, &output_path) {
            Ok(example) => {
                let input_text = format_input(&example.input);
                let target_text = format_target(&example.target.state_changes);
                let input_ids = tokenizer.encode(&input_text, false)?;
                let target_ids = tokenizer.encode(&target_text, false)?;
                examples.push(TrainingExample { input_ids, target_ids });
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
    batch_indices: &[usize],
    eos_id: u32,
    device: &Device,
) -> anyhow::Result<(Tensor, Vec<usize>, Vec<usize>)> {
    let mut batch_ids: Vec<Vec<u32>> = Vec::new();
    let mut sep_positions: Vec<usize> = Vec::new();
    let mut actual_lengths: Vec<usize> = Vec::new();

    for &idx in batch_indices {
        let ex = &examples[idx];
        let sep_pos = ex.input_ids.len();

        let mut combined = ex.input_ids.clone();
        combined.push(eos_id); // separator between input and target
        combined.extend(&ex.target_ids);
        combined.push(eos_id);

        actual_lengths.push(combined.len());
        sep_positions.push(sep_pos);
        batch_ids.push(combined);
    }

    // pad to max length (flattened directly, no intermediate Vec-of-Vecs)
    let max_len = batch_ids.iter().map(|ids| ids.len()).max().unwrap_or(0);
    let mut flat = Vec::with_capacity(batch_ids.len() * max_len);
    for ids in &batch_ids {
        flat.extend_from_slice(ids);
        flat.resize(flat.len() + (max_len - ids.len()), eos_id);
    }
    let shape = (batch_ids.len(), max_len);
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
        let end = actual_lengths[bi].saturating_sub(1);
        assert!(end <= logit_len, "sequence {bi} exceeds model context window ({end} > {logit_len})");
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
                top5.sort_by(|a, b| b.0.total_cmp(&a.0));
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
    pub batch_size: Option<usize>,
    pub checkpoint_every: usize,
    pub output: std::path::PathBuf,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            epochs: 100,
            learning_rate: 0.001,
            batch_size: None,
            checkpoint_every: 0,
            output: std::path::PathBuf::from("model.safetensors"),
        }
    }
}

pub fn train(
    model: &Llama,
    varmap: &VarMap,
    examples: &[TrainingExample],
    eos_id: u32,
    device: &Device,
    config: &TrainConfig,
    llama_config: &llama::Config,
) -> anyhow::Result<()> {
    let vars = varmap.all_vars();
    let mut adam = AdamW::new_lr(vars.clone(), config.learning_rate)?;

    let batch_size = match config.batch_size {
        None => examples.len(),
        Some(bs) => bs.min(examples.len()),
    };

    // Fixed train/validation split (90/10)
    let val_size = (examples.len() / 10).max(1);
    let train_size = examples.len().saturating_sub(val_size);

    let mut split_rng = StdRng::seed_from_u64(42);
    let mut all_indices: Vec<usize> = (0..examples.len()).collect();
    all_indices.shuffle(&mut split_rng);
    let (train_indices, val_indices) = all_indices.split_at(train_size);

    let mut rng = rand::rng();
    let n_batches = (train_indices.len() + batch_size - 1) / batch_size;
    let total_steps = n_batches * config.epochs;
    let warmup_steps = total_steps / 10;

    eprintln!(
        "training: {} train / {} val examples across ~{} batches/epoch, {} epochs, lr={}",
        train_indices.len(),
        val_indices.len(),
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

    let mut global_step = 0usize;

    let mut epoch_indices: Vec<usize> = train_indices.to_vec();
    for epoch in 0..config.epochs {
        epoch_indices.shuffle(&mut rng);

        let mut epoch_loss = 0.0f64;
        let mut n_seen = 0;

        for batch_idx in 0..n_batches {
            let start = batch_idx * batch_size;
            let end = (start + batch_size).min(train_indices.len());
            let batch_indices = &epoch_indices[start..end];

            // LR schedule: linear warmup → cosine decay
            let lr = if global_step < warmup_steps {
                config.learning_rate * global_step as f64 / warmup_steps.max(1) as f64
            } else {
                let denom = total_steps.saturating_sub(warmup_steps).max(1);
                let progress = (global_step - warmup_steps) as f64 / denom as f64;
                config.learning_rate * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos())
            };
            adam.set_learning_rate(lr);

            let (batch_tensor, sep_positions, actual_lengths) =
                prepare_batch(examples, batch_indices, eos_id, device)?;
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

            epoch_loss += loss_val * batch_indices.len() as f64;
            n_seen += batch_indices.len();
            global_step += 1;
        }

        let avg_loss = epoch_loss / n_seen as f64;

        // Validation loss (no gradient)
        let val_loss = if val_indices.is_empty() {
            None
        } else {
            let (val_tensor, val_sep, val_lens) =
                prepare_batch(examples, val_indices, eos_id, device)?;
            let loss = loss_for_batch(
                model,
                varmap,
                &val_tensor,
                &val_sep,
                &val_lens,
                llama_config,
                device,
                false,
            )?;
            Some(loss.to_scalar::<f32>()? as f64)
        };

        match val_loss {
            Some(vl) => {
                pb.set_message(format!(
                    "epoch {:3}  train {:.6}  val {:.6}",
                    epoch + 1, avg_loss, vl
                ));
                eprintln!(
                    "Epoch: {:3} | train loss {:.6} | val loss {:.6}",
                    epoch + 1, avg_loss, vl
                );
            }
            None => {
                pb.set_message(format!("epoch {:3}  loss {:.6}", epoch + 1, avg_loss));
                eprintln!("Epoch: {:3} | loss {:.6}", epoch + 1, avg_loss);
            }
        }
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
