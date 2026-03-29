use std::path::{Path, PathBuf};

pub fn collect_page_images(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|path| is_rendered_page_png(path))
        .collect();
    paths.sort();
    Ok(paths)
}

fn is_rendered_page_png(path: &Path) -> bool {
    let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    filename.starts_with("page-") && filename.ends_with(".png")
}
