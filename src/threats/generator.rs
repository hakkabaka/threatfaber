use std::path::{Path, PathBuf};

use rig::{
    completion::CompletionModel,
    message::{AssistantContent, Message},
};

use super::prompt::{build_prompt, system_prompt};

pub const THREAT_MODEL_FILENAME: &str = "threat_model.md";

pub struct ThreatModelGenerator<M> {
    model: M,
}

impl<M: CompletionModel> ThreatModelGenerator<M> {
    pub fn new(model: M) -> Self {
        Self { model }
    }

    pub async fn generate_and_write(
        &self,
        system_model_path: &Path,
        output_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let system_model_json = std::fs::read_to_string(system_model_path)
            .map_err(|e| format!("failed to read {}: {e}", system_model_path.display()))?;
        let message = Message::user(build_prompt(&system_model_json));

        let response = self
            .model
            .completion_request(message)
            .preamble(system_prompt().to_string())
            .send()
            .await?;

        let markdown = response
            .choice
            .iter()
            .find_map(|content| match content {
                AssistantContent::Text(text) => Some(strip_code_fence(&text.text).trim()),
                _ => None,
            })
            .ok_or("LLM returned no text for threat model")?;

        let output_path = output_dir.join(THREAT_MODEL_FILENAME);
        std::fs::write(&output_path, normalize_markdown(markdown))?;
        Ok(output_path)
    }
}

fn strip_code_fence(s: &str) -> &str {
    let s = s.trim();
    let s = s.strip_prefix("```markdown").unwrap_or(s);
    let s = s.strip_prefix("```md").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    s.strip_suffix("```").unwrap_or(s).trim()
}

fn normalize_markdown(markdown: &str) -> String {
    let mut out = markdown.trim().to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}
