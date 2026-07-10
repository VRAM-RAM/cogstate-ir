use anyhow::Result;

use crate::progress_handler::HfProgress;
use crate::util;

pub struct TokenizerWrapper {
    inner: tokenizers::Tokenizer,
    pub eos_id: u32,
}

impl TokenizerWrapper {
    pub fn from_pretrained(model_id: &str) -> Result<Self> {
        let (owner, name) = util::split_model_id(model_id);
        let client = hf_hub::HFClientSync::new()?;
        let repo = client.model(owner, name);
        let path = repo
            .download_file()
            .filename("tokenizer.json".to_string())
            .maybe_progress(Some(hf_hub::progress::Progress::new(HfProgress::new(
                "tokenizer.json".to_string(),
            ))))
            .send()
            .map_err(|e| anyhow::anyhow!("downloading tokenizer.json: {e}"))?;
        Self::from_file(&path)
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let inner = tokenizers::Tokenizer::from_file(path)
            .map_err(|e| anyhow::anyhow!("loading tokenizer from {}: {e}", path.display()))?;
        let eos_id = inner
            .token_to_id("<|im_end|>")
            .or_else(|| inner.token_to_id("</s>"))
            .map(|id| id.into())
            .unwrap_or(2);
        Ok(Self { inner, eos_id })
    }

    pub fn encode(&self, text: &str, add_special: bool) -> Result<Vec<u32>> {
        let encoding = self
            .inner
            .encode(text, add_special)
            .map_err(|e| anyhow::anyhow!("tokenizer encode: {e}"))?;
        Ok(encoding.get_ids().to_vec())
    }

    pub fn decode(&self, ids: &[u32]) -> Result<String> {
        self.inner
            .decode(ids, true)
            .map_err(|e| anyhow::anyhow!("tokenizer decode: {e}"))
    }

    /// Returns the full vocabulary size, including any added special tokens.
    pub fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }
}
