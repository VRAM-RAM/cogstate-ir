use std::collections::BTreeMap;

use candle_core::{DType, Device, IndexOp, Tensor};
use candle_transformers::models::llama::{self, Llama};

use rand::Rng;

use crate::model;
use crate::spec::{Magnitude, MemoryAction, ReflectionAction, StateChanges, StateChangesInner};
use crate::tokenizer::TokenizerWrapper;

/// Generate pipe-format output from an input text using KV-cache decoding.
pub fn generate(
    model: &Llama,
    tokenizer: &TokenizerWrapper,
    input_text: &str,
    device: &Device,
    config: &llama::Config,
    max_len: usize,
    temperature: Option<f64>,
) -> anyhow::Result<String> {
    let mut cache = model::create_cache(true, DType::F32, config, device)?;

    // Encode the input prefix (no special tokens) + separator token
    let mut prompt_ids = tokenizer.encode(input_text, false)?;
    prompt_ids.push(tokenizer.eos_id); // separator

    let mut generated: Vec<u32> = Vec::new();

    // Prefill: run all prompt tokens at once
    let prompt_len = prompt_ids.len();
    let prompt_tensor = Tensor::from_slice(&prompt_ids, (1, prompt_len), device)?;
    let _logits = model.forward(&prompt_tensor, 0, &mut cache)?;

    for _ in 0..max_len {
        // Feed the last generated token (or last prompt token on first step)
        let last_id = if generated.is_empty() {
            prompt_ids[prompt_len - 1]
        } else {
            generated[generated.len() - 1]
        };
        let input_t = Tensor::from_slice(&[last_id], (1, 1), device)?;
        let logits = model.forward(&input_t, prompt_len + generated.len() - 1, &mut cache)?;

        let last_logits = logits.i(0)?;
        let next = match temperature {
            Some(t) if t > 0.0 => {
                let scaled = (&last_logits / t)?;
                let probs = candle_nn::ops::softmax(&scaled, 0)?;
                let mut rng = rand::rng();
                let mut cum = 0.0f64;
                let sample: f64 = rng.random();
                let mut token = 0u32;
                for (i, p) in probs.to_vec1::<f32>()?.into_iter().enumerate() {
                    cum += p as f64;
                    if sample <= cum {
                        token = i as u32;
                        break;
                    }
                }
                token
            }
            _ => last_logits.argmax(0)?.to_scalar::<u32>()?,
        };

        if next == tokenizer.eos_id {
            break;
        }

        generated.push(next);

        if generated.len() > 200 {
            break;
        }
    }

    Ok(tokenizer.decode(&generated)?)
}

/// Parse pipe-format text into StateChanges, converting to YAML-compatible format.
/// Logs a warning to stderr if any lines were unrecognized.
pub fn pipe_to_state_changes(pipe: &str) -> StateChanges {
    let lines: Vec<&str> = pipe.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    if lines.len() == 1 && lines[0] == "no_changes" {
        return StateChanges::NoChanges("no_changes".to_string());
    }

    let mut emotion: BTreeMap<String, Magnitude> = BTreeMap::new();
    let mut relationship: BTreeMap<String, Magnitude> = BTreeMap::new();
    let mut belief: BTreeMap<String, Magnitude> = BTreeMap::new();
    let mut memory: Option<MemoryAction> = None;
    let mut reflection: Option<ReflectionAction> = None;
    let mut dropped = 0usize;

    for line in lines {
        let parts: Vec<&str> = line.split('|').collect();
        match parts[0] {
            "emotion" if parts.len() >= 3 => {
                if let Some(mag) = str_to_mag(parts[2]) {
                    emotion.insert(parts[1].to_string(), mag);
                } else {
                    dropped += 1;
                }
            }
            "relationship" if parts.len() >= 3 => {
                if let Some(mag) = str_to_mag(parts[2]) {
                    relationship.insert(parts[1].to_string(), mag);
                } else {
                    dropped += 1;
                }
            }
            "belief" if parts.len() >= 3 => {
                if let Some(mag) = str_to_mag(parts[2]) {
                    belief.insert(parts[1].to_string(), mag);
                } else {
                    dropped += 1;
                }
            }
            "memory" if parts.len() >= 2 => {
                if parts[1] == "reinforce_previous_conflict" {
                    memory = Some(MemoryAction::ReinforcePreviousConflict);
                } else {
                    dropped += 1;
                }
            }
            "reflection" if parts.len() >= 2 => {
                if parts[1] == "required" {
                    reflection = Some(ReflectionAction::Required);
                } else {
                    dropped += 1;
                }
            }
            _ => dropped += 1,
        }
    }

    if dropped > 0 {
        eprintln!("warning: pipe_to_state_changes dropped {dropped} unrecognized line(s)");
    }

    StateChanges::Record(StateChangesInner {
        emotion: if emotion.is_empty() { None } else { Some(emotion) },
        relationship: if relationship.is_empty() {
            None
        } else {
            Some(relationship)
        },
        belief: if belief.is_empty() { None } else { Some(belief) },
        memory,
        reflection,
    })
}

fn str_to_mag(s: &str) -> Option<Magnitude> {
    match s {
        "increases_a_lot" => Some(Magnitude::IncreasesALot),
        "increases" => Some(Magnitude::Increases),
        "increases_a_little" => Some(Magnitude::IncreasesALittle),
        "decreases_a_little" => Some(Magnitude::DecreasesALittle),
        "decreases" => Some(Magnitude::Decreases),
        "decreases_a_lot" => Some(Magnitude::DecreasesALot),
        _ => None,
    }
}




