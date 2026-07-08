use std::path::Path;

use candle_core::{DType, Device, Result};
use candle_nn::{VarBuilder, VarMap};
use candle_transformers::models::llama::{self, Cache, Llama, LlamaConfig};

use crate::progress_handler::HfProgress;

pub fn config_from_json(path: &Path) -> anyhow::Result<llama::Config> {
    let content = std::fs::read_to_string(path)?;
    let hf: LlamaConfig = serde_json::from_str(&content)?;
    Ok(hf.into_config(/*use_flash_attn=*/ false))
}

fn split_model_id(id: &str) -> (&str, &str) {
    id.split_once('/').unwrap_or(("", id))
}

/// Build model from HuggingFace safetensors into a VarMap for fine-tuning.
pub fn build_model(
    model_id: &str,
    varmap: &mut VarMap,
    device: &Device,
) -> anyhow::Result<(Llama, llama::Config)> {
    let (owner, name) = split_model_id(model_id);
    let client = hf_hub::HFClientSync::new()?;
    let repo = client.model(owner, name);

    let config_path = repo
        .download_file()
        .filename("config.json".to_string())
        .maybe_progress(Some(hf_hub::progress::Progress::new(HfProgress::new(
            "config.json".to_string(),
        ))))
        .send()?;
    let weights_path = repo
        .download_file()
        .filename("model.safetensors".to_string())
        .maybe_progress(Some(hf_hub::progress::Progress::new(HfProgress::new(
            "model.safetensors".to_string(),
        ))))
        .send()?;

    let cfg = config_from_json(&config_path)?;

    // Step 1: Build model first (creates random vars in VarMap with requires_grad=true).
    eprintln!("building model…");
    let vb = VarBuilder::from_varmap(varmap, DType::F32, device);
    let model = Llama::load(vb, &cfg)?;

    // Step 2: Overwrite random vars with pretrained weights from safetensors.
    // set_one requires the var to already exist; we build the model first so it does.
    // We explicitly convert BF16→F32 since var.set() doesn't handle dtype conversion.
    eprintln!("loading pre-trained weights…");
    let safetensors = candle_core::safetensors::load(&weights_path, device)?;
    for (name, t) in &safetensors {
        let converted = t.to_dtype(DType::F32)?;
        varmap.set_one(name, &converted)?;
    }

    Ok((model, cfg))
}

/// Reload a fine-tuned model, reading weights from a safetensors file.
pub fn load_model(
    varmap: &mut VarMap,
    weights_path: &Path,
    config: &llama::Config,
    device: &Device,
) -> Result<Llama> {
    // Build model first (creates vars in VarMap), then load saved weights.
    let vb = VarBuilder::from_varmap(varmap, DType::F32, device);
    let model = Llama::load(vb, config)?;
    varmap.load(weights_path)?;
    Ok(model)
}

pub fn save_model(varmap: &VarMap, path: &Path) -> Result<()> {
    varmap.save(path)
}

pub fn create_cache(use_kv_cache: bool, dtype: DType, config: &llama::Config, device: &Device) -> Result<Cache> {
    Cache::new(use_kv_cache, dtype, config, device)
}

/// Dump aggregate weight stats for diagnostics.
pub fn dump_weight_stats(varmap: &VarMap) {
    let vars = varmap.all_vars();
    let total_params: usize = vars.iter().map(|v| v.shape().elem_count()).sum();
    eprintln!("  weights: {} tensors, {} total params", vars.len(), total_params);
}
