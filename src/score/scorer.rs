use std::path::{Path, PathBuf};

use base64::{Engine, engine::general_purpose::STANDARD};
use rig::{
    OneOrMany,
    completion::CompletionModel,
    message::{AssistantContent, ImageDetail, ImageMediaType, Message, UserContent},
};
use serde::Deserialize;

use super::{
    images::collect_page_images,
    prompt::{build_prompt, system_prompt},
    read_preferred_markdown,
    types::{DocumentScore, ImageScore},
};

pub struct Scorer<M> {
    model: M,
}

impl<M: CompletionModel> Scorer<M> {
    pub fn new(model: M) -> Self {
        Self { model }
    }

    /// Score a single extracted document directory.
    ///
    /// Sends the markdown content and all images together in one request so the
    /// LLM has full context (an image may annotate concepts described in the
    /// text), but returns separate scores for the text and each image.
    pub async fn score_document(
        &self,
        doc_dir: &Path,
    ) -> Result<DocumentScore, Box<dyn std::error::Error>> {
        let markdown = read_preferred_markdown(doc_dir)?;

        let image_paths = collect_page_images(doc_dir)?;

        let mut content = vec![UserContent::text(build_prompt(&markdown, &image_paths))];
        for path in &image_paths {
            let bytes = std::fs::read(path)?;
            let b64 = STANDARD.encode(&bytes);
            content.push(UserContent::image_base64(
                b64,
                Some(ImageMediaType::PNG),
                Some(ImageDetail::Auto),
            ));
        }

        let message = Message::User {
            content: OneOrMany::many(content).map_err(|_| "scoring content cannot be empty")?,
        };

        let response = self
            .model
            .completion_request(message)
            .preamble(system_prompt().to_string())
            .send()
            .await?;

        let text = response
            .choice
            .iter()
            .find_map(|c| {
                if let AssistantContent::Text(t) = c {
                    Some(t.text.as_str())
                } else {
                    None
                }
            })
            .ok_or("LLM returned no text")?;

        let raw: RawDocumentScore = serde_json::from_str(strip_code_fence(text))
            .map_err(|e| format!("failed to parse LLM JSON: {e}\nresponse was: {text}"))?;

        Ok(normalize_scores(raw, &image_paths))
    }
}

#[derive(Debug, Deserialize)]
struct RawImageScore {
    filename: String,
    score: i16,
}

#[derive(Debug, Deserialize)]
struct RawDocumentScore {
    text_score: i16,
    #[serde(default)]
    images: Vec<RawImageScore>,
}

fn strip_code_fence(s: &str) -> &str {
    let s = s.trim();
    let s = s.strip_prefix("```json").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    s.strip_suffix("```").unwrap_or(s).trim()
}

fn normalize_scores(raw: RawDocumentScore, image_paths: &[PathBuf]) -> DocumentScore {
    let images = image_paths
        .iter()
        .map(|path| {
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let score = raw
                .images
                .iter()
                .find(|img| img.filename == filename)
                .map(|img| clamp_score(img.score))
                .unwrap_or(0);

            ImageScore { filename, score }
        })
        .collect();

    DocumentScore {
        text_score: clamp_score(raw.text_score),
        images,
    }
}

fn clamp_score(score: i16) -> u8 {
    score.clamp(0, 100) as u8
}
