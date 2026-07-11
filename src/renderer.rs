use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Configuration for the renderer (llama-server backend).
pub struct RendererConfig {
    pub model_path: String,
    pub port: u16,
}

/// Manages a llama-server child process and provides an HTTP client.
pub struct Renderer {
    child: Option<Child>,
    port: u16,
}

impl Renderer {
    /// Start llama-server as a child process, waiting until it responds to health checks.
    pub fn start(config: &RendererConfig) -> anyhow::Result<Self> {
        let child = Command::new("llama-server")
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(config.port.to_string())
            .arg("-m")
            .arg(&config.model_path)
            .arg("--no-webui")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| anyhow::anyhow!(
                "failed to start llama-server: {e}. Is llama.cpp installed? Try: brew install llama.cpp"
            ))?;

        let renderer = Self {
            child: Some(child),
            port: config.port,
        };

        renderer.wait_for_ready()?;
        Ok(renderer)
    }

    /// Poll the health endpoint until the server responds.
    fn wait_for_ready(&self) -> anyhow::Result<()> {
        let url = format!("http://127.0.0.1:{}/health", self.port);
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(120);

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("llama-server did not become ready within 120 seconds");
            }
            match ureq::get(&url).call() {
                Ok(resp) if resp.status() == 200 => return Ok(()),
                _ => std::thread::sleep(Duration::from_millis(500)),
            }
        }
    }

    /// Generate a response from the renderer model.
    /// `messages` uses the OpenAI chat format: `{role: "system"|"user"|"assistant", content: "..."}`.
    pub fn generate(&self, messages: &[ChatMessage]) -> anyhow::Result<String> {
        let url = format!("http://127.0.0.1:{}/v1/chat/completions", self.port);

        let body = serde_json::json!({
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 512,
            "stream": false,
        });

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&body)
            .map_err(|e| anyhow::anyhow!("renderer request failed: {e}"))?;

        let data: serde_json::Value = response
            .into_json()
            .map_err(|e| anyhow::anyhow!("renderer response parse failed: {e}"))?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "unexpected renderer response: missing choices[0].message.content"
                )
            })?;

        Ok(content.to_string())
    }

    /// Stop the renderer server.
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// A chat message compatible with the OpenAI chat completions format.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Build the system prompt from a character state.
pub fn build_system_prompt(state: &crate::engine::CharacterState) -> String {
    let mut parts = Vec::new();

    if !state.personality.is_empty() {
        parts.push(format!("Personality: {}", state.personality.join(", ")));
    }

    if !state.emotions.is_empty() {
        let emo: Vec<String> = state
            .emotions
            .iter()
            .map(|(k, v)| format!("{}: {:.2}", k, v))
            .collect();
        parts.push(format!("Emotional state: {}", emo.join(", ")));
    }

    for (target, traits) in &state.relationships {
        let t: Vec<String> = traits
            .iter()
            .map(|(k, v)| format!("{}: {:.2}", k, v))
            .collect();
        parts.push(format!("Relationship with {}: {}", target, t.join(", ")));
    }

    if !state.beliefs.is_empty() {
        let b: Vec<String> = state
            .beliefs
            .iter()
            .map(|(k, v)| format!("{}: {:.2}", k, v))
            .collect();
        parts.push(format!("Beliefs: {}", b.join(", ")));
    }

    if !state.memory.is_empty() {
        let m: Vec<String> = state
            .memory
            .iter()
            .map(|e| format!("{} (strength: {:.2})", e.id, e.strength))
            .collect();
        parts.push(format!("Memory: {}", m.join(", ")));
    }

    let state_summary = parts.join("\n");

    format!(
        r#"You are roleplaying as a character with the following attributes.

{state_summary}

Respond as this character, expressing their personality, emotions, and beliefs through your words and tone. Do not describe your internal state — just speak naturally as the character would."#
    )
}
