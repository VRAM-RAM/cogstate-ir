use std::collections::BTreeMap;
use std::path::Path;

use crate::engine::{self, CharacterState};
use crate::model;
use crate::predict;
use crate::spec;
use crate::tokenizer::TokenizerWrapper;

fn value_to_label(v: f32) -> &'static str {
    if v < 0.2 {
        "very_low"
    } else if v < 0.4 {
        "low"
    } else if v < 0.6 {
        "medium"
    } else if v < 0.8 {
        "high"
    } else {
        "very_high"
    }
}

pub fn state_to_input(
    state: &CharacterState,
    user_message: &str,
    previous_message: Option<&str>,
) -> spec::Input {
    let personality = state.personality.clone();

    let relationship: BTreeMap<String, BTreeMap<String, String>> = state
        .relationships
        .iter()
        .map(|(target, traits)| {
            let mapped = traits
                .iter()
                .map(|(k, v)| (k.clone(), value_to_label(*v).to_string()))
                .collect();
            (target.clone(), mapped)
        })
        .collect();

    let current_state: BTreeMap<String, String> = state
        .emotions
        .iter()
        .map(|(name, v)| (name.clone(), value_to_label(*v).to_string()))
        .collect();

    spec::Input {
        character: spec::CharacterSection { personality },
        relationship,
        current_state,
        previous_character_message: previous_message.map(|s| s.to_string()),
        user_message: user_message.to_string(),
    }
}

pub fn run(
    state_path: &Path,
    user_message: &str,
    previous_message: Option<&str>,
    weights: &Path,
    model_id: &str,
    output: Option<&Path>,
    no_metal: bool,
) -> anyhow::Result<()> {
    let device = if no_metal {
        candle_core::Device::Cpu
    } else if candle_core::utils::metal_is_available() {
        candle_core::Device::new_metal(0)?
    } else {
        candle_core::Device::Cpu
    };
    println!("device: {device:?}");

    // Load character state
    let state_content = std::fs::read_to_string(state_path)?;
    let state = CharacterState::from_json(&state_content)?;
    println!("loaded character state from {}", state_path.display());

    // Build compiler input from state
    let input = state_to_input(&state, user_message, previous_message);
    let input_text = crate::train::format_input(&input);

    // Load model
    let tokenizer = TokenizerWrapper::from_pretrained(model_id)?;
    let llama_config = super::download_config(model_id)?;
    let mut varmap = candle_nn::VarMap::new();
    let model = model::load_model(&mut varmap, weights, &llama_config, &device)?;

    // Run compiler
    let raw_output = predict::generate(
        &model,
        &tokenizer,
        &input_text,
        &device,
        &llama_config,
        256,
        None,
    )?;

    // Parse IR ops
    let state_changes = predict::pipe_to_state_changes(&raw_output);
    let target = spec::Target {
        state_changes: state_changes.clone(),
    };
    let yaml = serde_yaml::to_string(&target)?;

    println!("\npredicted_ir:\n{}", yaml);

    // Apply to state engine
    let ir_ops = engine::ops_from_state_changes(&state_changes);
    let new_state = engine::Engine::apply_state(&state, &ir_ops);

    let new_json = new_state.to_json()?;

    match output {
        Some(path) => {
            std::fs::write(path, &new_json)?;
            println!("new_state written to {}", path.display());
        }
        None => {
            println!("\nnew_state:\n{new_json}");
        }
    }

    Ok(())
}
