use std::path::{Path, PathBuf};

use rig::{
    completion::CompletionModel,
    message::{AssistantContent, Message},
};

use super::prompt::{build_cleaning_prompt, cleaning_system_prompt};

pub const CLEANED_MARKDOWN_FILENAME: &str = "cleaned_content.md";

pub struct Cleaner<M> {
    model: M,
}

impl<M: CompletionModel> Cleaner<M> {
    pub fn new(model: M) -> Self {
        Self { model }
    }

    pub async fn clean_document(
        &self,
        doc_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let source_path = doc_dir.join("content.md");
        let markdown = std::fs::read_to_string(&source_path)
            .map_err(|e| format!("failed to read {}: {e}", source_path.display()))?;

        let message = Message::user(build_cleaning_prompt(&markdown));

        let response = self
            .model
            .completion_request(message)
            .preamble(cleaning_system_prompt().to_string())
            .send()
            .await?;

        let cleaned = response
            .choice
            .iter()
            .find_map(|content| match content {
                AssistantContent::Text(text) => Some(strip_code_fence(&text.text).trim()),
                _ => None,
            })
            .ok_or("LLM returned no text for cleaned document")?;

        let cleaned_path = doc_dir.join(CLEANED_MARKDOWN_FILENAME);
        std::fs::write(&cleaned_path, normalize_cleaned_markdown(cleaned))?;

        Ok(cleaned_path)
    }
}

pub fn cleaned_markdown_path(doc_dir: &Path) -> PathBuf {
    doc_dir.join(CLEANED_MARKDOWN_FILENAME)
}

pub fn preferred_markdown_path(doc_dir: &Path) -> PathBuf {
    let cleaned_path = cleaned_markdown_path(doc_dir);
    if cleaned_path.is_file() {
        cleaned_path
    } else {
        doc_dir.join("content.md")
    }
}

pub fn read_preferred_markdown(doc_dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let markdown_path = preferred_markdown_path(doc_dir);
    std::fs::read_to_string(&markdown_path)
        .map_err(|e| format!("failed to read {}: {e}", markdown_path.display()).into())
}

fn strip_code_fence(s: &str) -> &str {
    let s = s.trim();
    let s = s.strip_prefix("```markdown").unwrap_or(s);
    let s = s.strip_prefix("```md").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    s.strip_suffix("```").unwrap_or(s).trim()
}

fn normalize_cleaned_markdown(cleaned: &str) -> String {
    if cleaned.eq_ignore_ascii_case("NONE")
        || cleaned.eq_ignore_ascii_case("NO_RELEVANT_ARCHITECTURE")
    {
        String::new()
    } else {
        let mut out = cleaned.trim().to_string();
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }
}
