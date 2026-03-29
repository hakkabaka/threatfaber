use serde::{Deserialize, Serialize};

pub const SCORE_THRESHOLD: u8 = 70;

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageScore {
    pub filename: String,
    pub score: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DocumentScore {
    pub text_score: u8,
    #[serde(default)]
    pub images: Vec<ImageScore>,
}

impl DocumentScore {
    pub fn text_relevant(&self) -> bool {
        self.text_score >= SCORE_THRESHOLD
    }

    pub fn overall_score(&self) -> u8 {
        self.images
            .iter()
            .map(|img| img.score)
            .max()
            .unwrap_or(self.text_score)
            .max(self.text_score)
    }

    pub fn relevant(&self) -> bool {
        self.overall_score() >= SCORE_THRESHOLD
    }

    pub fn relevant_images(&self) -> impl Iterator<Item = &ImageScore> {
        self.images
            .iter()
            .filter(|img| img.score >= SCORE_THRESHOLD)
    }
}
