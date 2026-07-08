use anyhow::Result;

use crate::progress_handler::HfProgress;

pub struct TokenizerWrapper {
    inner: tokenizers::Tokenizer,
    pub bos_id: u32,
    pub eos_id: u32,
}

fn split_model_id(id: &str) -> (&str, &str) {
    id.split_once('/').unwrap_or(("", id))
}

impl TokenizerWrapper {
    pub fn from_pretrained(model_id: &str) -> Result<Self> {
        let (owner, name) = split_model_id(model_id);
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
        let bos_id = inner
            .token_to_id("<|im_start|>")
            .map(|id| id.into())
            .unwrap_or(2);
        let eos_id = inner
            .token_to_id("<|im_end|>")
            .map(|id| id.into())
            .unwrap_or(2);
        Ok(Self { inner, bos_id, eos_id })
    }

    pub fn encode(&self, text: &str, add_special: bool) -> Vec<u32> {
        let encoding = self.inner.encode(text, add_special).unwrap();
        encoding.get_ids().to_vec()
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        self.inner.decode(ids, true).unwrap()
    }

    pub fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }
}
