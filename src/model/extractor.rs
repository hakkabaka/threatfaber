use std::path::{Path, PathBuf};

use rig::{
    completion::CompletionModel,
    message::{AssistantContent, Message},
};

use crate::score::read_preferred_markdown;

use super::{
    prompt::{build_prompt, system_prompt},
    types::SystemModel,
};

pub const SYSTEM_MODEL_FILENAME: &str = "system_model.json";

pub struct SystemModelExtractor<M> {
    model: M,
}

impl<M: CompletionModel> SystemModelExtractor<M> {
    pub fn new(model: M) -> Self {
        Self { model }
    }

    pub async fn extract_service(
        &self,
        doc_dirs: &[PathBuf],
    ) -> Result<SystemModel, Box<dyn std::error::Error>> {
        let markdown = build_service_markdown(doc_dirs)?;
        self.extract_from_markdown(&markdown).await
    }

    pub async fn extract_service_and_write(
        &self,
        doc_dirs: &[PathBuf],
        output_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let system_model = self.extract_service(doc_dirs).await?;
        let output_path = output_dir.join(SYSTEM_MODEL_FILENAME);
        std::fs::write(&output_path, serde_json::to_string_pretty(&system_model)?)?;
        Ok(output_path)
    }

    async fn extract_from_markdown(
        &self,
        markdown: &str,
    ) -> Result<SystemModel, Box<dyn std::error::Error>> {
        let message = Message::user(build_prompt(markdown));

        let response = self
            .model
            .completion_request(message)
            .preamble(system_prompt().to_string())
            .send()
            .await?;

        let text = response
            .choice
            .iter()
            .find_map(|content| match content {
                AssistantContent::Text(text) => Some(text.text.as_str()),
                _ => None,
            })
            .ok_or("LLM returned no text for system model")?;

        serde_json::from_str(strip_code_fence(text)).map_err(|e| {
            format!("failed to parse system model JSON: {e}\nresponse was: {text}").into()
        })
    }
}

fn build_service_markdown(doc_dirs: &[PathBuf]) -> Result<String, Box<dyn std::error::Error>> {
    let mut sections = Vec::new();

    for doc_dir in doc_dirs {
        let markdown = read_preferred_markdown(doc_dir)?;
        let doc_name = doc_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        sections.push(format!("## Source: {doc_name}\n\n{}", markdown.trim()));
    }

    Ok(sections.join("\n\n"))
}

fn strip_code_fence(s: &str) -> &str {
    let s = s.trim();
    let s = s.strip_prefix("```json").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    s.strip_suffix("```").unwrap_or(s).trim()
}
