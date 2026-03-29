use std::path::PathBuf;
use unpdf::{parse_file, render};

use super::pdfium::render_pages_to_png;

pub struct Pdf {
    path: PathBuf,
    output_dir: PathBuf,
    doc_id: String,
}

impl Pdf {
    pub fn new(path: PathBuf, output_dir: PathBuf, doc_id: String) -> Self {
        Self {
            path,
            output_dir,
            doc_id,
        }
    }

    pub fn extract(&self) -> Result<(), Box<dyn std::error::Error>> {
        let doc = parse_file(&self.path)
            .map_err(|e| format!("failed to parse {}: {e}", self.path.display()))?;

        let doc_dir = self.output_dir.join("extract").join(&self.doc_id);
        std::fs::create_dir_all(&doc_dir)?;

        let options = render::RenderOptions::default();
        let markdown = render::to_markdown(&doc, &options)?;
        std::fs::write(doc_dir.join("content.md"), markdown)?;

        render_pages_to_png(&self.path, &doc_dir)?;

        Ok(())
    }
}
