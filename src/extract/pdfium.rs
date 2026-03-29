use std::path::{Path, PathBuf};

use pdfium_render::prelude::*;

const PAGE_RENDER_WIDTH: i32 = 1600;

pub fn render_pages_to_png(
    pdf_path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let bindings = bind_pdfium()
        .map_err(|e| format!("failed to bind pdfium for {}: {e}", pdf_path.display()))?;
    let pdfium = Pdfium::new(bindings);
    let document = pdfium
        .load_pdf_from_file(pdf_path, None)
        .map_err(|e| format!("failed to open {} with pdfium: {e}", pdf_path.display()))?;

    for (index, page) in document.pages().iter().enumerate() {
        let render = page.render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(PAGE_RENDER_WIDTH)
                .render_form_data(true),
        )?;
        let image = render.as_image();
        let filename = format!("page-{:03}.png", index + 1);
        image.save(output_dir.join(filename))?;
    }

    Ok(())
}

fn bind_pdfium() -> Result<Box<dyn PdfiumLibraryBindings>, PdfiumError> {
    for path in pdfium_library_candidates() {
        if let Ok(bindings) = Pdfium::bind_to_library(&path) {
            return Ok(bindings);
        }
    }

    Pdfium::bind_to_system_library()
}

fn pdfium_library_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = std::env::var("PDFIUM_LIB_PATH") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            candidates.push(PathBuf::from(trimmed));
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join(Pdfium::pdfium_platform_library_name()));
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(Pdfium::pdfium_platform_library_name()));
    }

    candidates
}
