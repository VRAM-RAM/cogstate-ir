use std::collections::BTreeMap;

use candle_core::{DType, Device, IndexOp, Tensor};
use candle_transformers::models::llama::{self, Llama};

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
) -> anyhow::Result<String> {
    let mut cache = model::create_cache(true, DType::F32, config, device)?;

    // Encode the input prefix (no special tokens) + separator token
    let mut prompt_ids = tokenizer.encode(input_text, false);
    prompt_ids.push(tokenizer.bos_id); // separator

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

        // Greedy: pick the highest probability token
        let last_logits = logits.i(0)?;
        let next = last_logits.argmax(0)?.to_scalar::<u32>()?;

        if next == tokenizer.eos_id {
            break;
        }

        generated.push(next);

        if generated.len() > 200 {
            break;
        }
    }

    Ok(tokenizer.decode(&generated))
}

/// Parse pipe-format text into StateChanges, converting to YAML-compatible format
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

    for line in lines {
        let parts: Vec<&str> = line.split('|').collect();
        match parts[0] {
            "emotion" if parts.len() >= 3 => {
                if let Some(mag) = str_to_mag(parts[2]) {
                    emotion.insert(parts[1].to_string(), mag);
                }
            }
            "relationship" if parts.len() >= 3 => {
                if let Some(mag) = str_to_mag(parts[2]) {
                    relationship.insert(parts[1].to_string(), mag);
                }
            }
            "belief" if parts.len() >= 3 => {
                if let Some(mag) = str_to_mag(parts[2]) {
                    belief.insert(parts[1].to_string(), mag);
                }
            }
            "memory" if parts.len() >= 2 => {
                if parts[1] == "reinforce_previous_conflict" {
                    memory = Some(MemoryAction::ReinforcePreviousConflict);
                }
            }
            "reflection" if parts.len() >= 2 => {
                if parts[1] == "required" {
                    reflection = Some(ReflectionAction::Required);
                }
            }
            _ => {}
        }
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

/// Convert StateChanges to a YAML string matching the user's output format
pub fn state_changes_to_yaml(changes: &StateChanges) -> String {
    match changes {
        StateChanges::NoChanges(_) => {
            return "state_changes:\n  no_changes\n".to_string();
        }
        StateChanges::Record(inner) => {
            let mut out = String::from("state_changes:\n");

            if let Some(emotions) = &inner.emotion {
                out.push_str("  emotion:\n");
                for (name, mag) in emotions {
                    out.push_str(&format!("    {name}: {}\n", mag_to_yaml_str(*mag)));
                }
            }

            if let Some(traits) = &inner.relationship {
                out.push_str("  relationship:\n");
                for (trait_name, mag) in traits {
                    out.push_str(&format!("    {trait_name}: {}\n", mag_to_yaml_str(*mag)));
                }
            }

            if let Some(beliefs) = &inner.belief {
                out.push_str("  belief:\n");
                for (id, mag) in beliefs {
                    out.push_str(&format!("    {id}: {}\n", mag_to_yaml_str(*mag)));
                }
            }

            if let Some(memory) = &inner.memory {
                match memory {
                    MemoryAction::ReinforcePreviousConflict => {
                        out.push_str("  memory: reinforce_previous_conflict\n");
                    }
                }
            }

            if let Some(reflection) = &inner.reflection {
                match reflection {
                    ReflectionAction::Required => {
                        out.push_str("  reflection: required\n");
                    }
                }
            }

            out
        }
    }
}

fn mag_to_yaml_str(m: Magnitude) -> &'static str {
    match m {
        Magnitude::IncreasesALot => "increases_a_lot",
        Magnitude::Increases => "increases",
        Magnitude::IncreasesALittle => "increases_a_little",
        Magnitude::DecreasesALittle => "decreases_a_little",
        Magnitude::Decreases => "decreases",
        Magnitude::DecreasesALot => "decreases_a_lot",
    }
}
