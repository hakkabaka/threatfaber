mod extract;
mod model;
mod score;
mod threats;

use std::path::{Path, PathBuf};

use clap::Parser;
use extract::pdf::Pdf;
use model::SystemModelExtractor;
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::openai;
use score::{Cleaner, DocumentScore, SCORE_THRESHOLD, Scorer};
use serde::Serialize;
use threats::ThreatModelGenerator;

const CLEANER_MODEL: &str = "gpt-5-nano";
const SCORER_MODEL: &str = "gpt-5-mini";
const SYSTEM_MODEL_MODEL: &str = "gpt-5.2";
const THREAT_MODEL_MODEL: &str = "gpt-5.2";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Folder containing one or more PDF files
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory
    #[arg(short, long, default_value = "result")]
    output: PathBuf,

    /// Minimum document relevance score to keep a document
    #[arg(long, default_value_t = SCORE_THRESHOLD)]
    min_score: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let pdf_inputs = collect_pdf_inputs(&args.input)?;

    if pdf_inputs.is_empty() {
        eprintln!("no PDF files found in {}", args.input.display());
        return Ok(());
    }

    std::fs::create_dir_all(&args.output)?;

    for pdf_input in &pdf_inputs {
        let pdf = Pdf::new(
            pdf_input.path.clone(),
            args.output.clone(),
            pdf_input.doc_id.clone(),
        );
        pdf.extract()?;
        println!("extracted: {}", pdf_input.relative_path.display());
    }

    let cleaner_model_name = model_name_from_env("THREATFABER_CLEANER_MODEL", CLEANER_MODEL);
    let scorer_model_name = model_name_from_env("THREATFABER_SCORER_MODEL", SCORER_MODEL);
    let system_model_name = model_name_from_env("THREATFABER_SYSTEM_MODEL", SYSTEM_MODEL_MODEL);
    let threat_model_name = model_name_from_env("THREATFABER_THREAT_MODEL", THREAT_MODEL_MODEL);

    let cleaner_model = openai::CompletionsClient::from_env().completion_model(&cleaner_model_name);
    let system_model_model =
        openai::CompletionsClient::from_env().completion_model(&system_model_name);
    let scorer_model = openai::CompletionsClient::from_env().completion_model(&scorer_model_name);
    let threat_model_model =
        openai::CompletionsClient::from_env().completion_model(&threat_model_name);
    let cleaner = Cleaner::new(cleaner_model);
    let system_model_extractor = SystemModelExtractor::new(system_model_model);
    let scorer = Scorer::new(scorer_model);
    let threat_model_generator = ThreatModelGenerator::new(threat_model_model);
    let mut summaries = Vec::with_capacity(pdf_inputs.len());
    let mut cleaned_doc_dirs = Vec::new();

    for pdf_input in &pdf_inputs {
        let doc_dir = args.output.join("extract").join(&pdf_input.doc_id);

        print!("cleaning: {} ... ", pdf_input.relative_path.display());
        match cleaner.clean_document(&doc_dir).await {
            Ok(path) => {
                println!(
                    "saved {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                cleaned_doc_dirs.push(doc_dir.clone());
            }
            Err(e) => {
                eprintln!("error cleaning {}: {e}", pdf_input.relative_path.display());
                continue;
            }
        }

        print!("scoring: {} ... ", pdf_input.relative_path.display());
        match scorer.score_document(&doc_dir).await {
            Ok(score) => {
                print_score_summary(&pdf_input.relative_path.display().to_string(), &score);
                summaries.push(DocumentSummary::from_score(
                    pdf_input,
                    score,
                    args.min_score,
                ));
            }
            Err(e) => {
                eprintln!("error scoring {}: {e}", pdf_input.relative_path.display());
            }
        }
    }

    if !cleaned_doc_dirs.is_empty() {
        print!("modeling service ... ");
        let system_model_path = match system_model_extractor
            .extract_service_and_write(&cleaned_doc_dirs, &args.output)
            .await
        {
            Ok(path) => {
                println!(
                    "saved {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                path
            }
            Err(e) => {
                eprintln!("error extracting service system model: {e}");
                write_results(&args.output, &summaries)?;
                print_run_summary(&summaries, args.min_score);
                return Ok(());
            }
        };

        print!("threat modeling service ... ");
        match threat_model_generator
            .generate_and_write(&system_model_path, &args.output)
            .await
        {
            Ok(path) => {
                println!(
                    "saved {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
            Err(e) => {
                eprintln!("error generating threat model: {e}");
            }
        }
    }

    write_results(&args.output, &summaries)?;
    print_run_summary(&summaries, args.min_score);

    Ok(())
}

fn model_name_from_env(var_name: &str, default: &str) -> String {
    std::env::var(var_name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn print_score_summary(name: &str, score: &DocumentScore) {
    println!();
    let relevance = if score.relevant() {
        "RELEVANT"
    } else {
        "filtered"
    };
    println!(
        "  overall [{:>3}/100] {relevance}  —  {name}",
        score.overall_score()
    );
    let relevance = if score.text_relevant() {
        "RELEVANT"
    } else {
        "filtered"
    };
    println!(
        "  text [{:>3}/100] {relevance}  —  {name}",
        score.text_score
    );

    for img in &score.images {
        let relevance = if img.score >= SCORE_THRESHOLD {
            "RELEVANT"
        } else {
            "filtered"
        };
        println!(
            "  img  [{:>3}/100] {relevance}  —  {}",
            img.score, img.filename
        );
    }
}

#[derive(Debug)]
struct PdfInput {
    path: PathBuf,
    relative_path: PathBuf,
    doc_id: String,
}

#[derive(Debug, Serialize)]
struct DocumentSummary {
    source_path: String,
    doc_id: String,
    text_score: u8,
    overall_score: u8,
    kept: bool,
    relevant_images: Vec<String>,
}

impl DocumentSummary {
    fn from_score(pdf_input: &PdfInput, score: DocumentScore, min_score: u8) -> Self {
        Self {
            source_path: pdf_input.relative_path.display().to_string(),
            doc_id: pdf_input.doc_id.clone(),
            text_score: score.text_score,
            overall_score: score.overall_score(),
            kept: score.overall_score() >= min_score,
            relevant_images: score
                .relevant_images()
                .map(|img| img.filename.clone())
                .collect(),
        }
    }
}

fn collect_pdf_inputs(input_root: &Path) -> Result<Vec<PdfInput>, Box<dyn std::error::Error>> {
    let mut stack = vec![input_root.to_path_buf()];
    let mut pdfs = Vec::new();

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if !file_type.is_file()
                || !path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
            {
                continue;
            }

            let relative_path = path.strip_prefix(input_root).unwrap_or(&path).to_path_buf();
            let doc_id = build_doc_id(&relative_path);

            pdfs.push(PdfInput {
                path,
                relative_path,
                doc_id,
            });
        }
    }

    pdfs.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(pdfs)
}

fn build_doc_id(path: &Path) -> String {
    let mut out = String::with_capacity(path.as_os_str().len());
    for ch in path.to_string_lossy().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }

    out.trim_matches('_').to_string()
}

fn write_results(
    output_dir: &Path,
    summaries: &[DocumentSummary],
) -> Result<(), Box<dyn std::error::Error>> {
    let kept: Vec<&DocumentSummary> = summaries.iter().filter(|doc| doc.kept).collect();
    let filtered: Vec<&DocumentSummary> = summaries.iter().filter(|doc| !doc.kept).collect();

    std::fs::write(
        output_dir.join("document_scores.json"),
        serde_json::to_string_pretty(summaries)?,
    )?;
    std::fs::write(
        output_dir.join("relevant_documents.json"),
        serde_json::to_string_pretty(&kept)?,
    )?;
    std::fs::write(
        output_dir.join("filtered_out_documents.json"),
        serde_json::to_string_pretty(&filtered)?,
    )?;

    Ok(())
}

fn print_run_summary(summaries: &[DocumentSummary], min_score: u8) {
    let kept = summaries.iter().filter(|doc| doc.kept).count();
    let filtered = summaries.len().saturating_sub(kept);

    println!();
    println!(
        "kept {kept}/{} documents with overall relevance >= {min_score}",
        summaries.len()
    );
    println!(
        "results written to document_scores.json, relevant_documents.json, and filtered_out_documents.json"
    );
    if filtered > 0 {
        println!("filtered out {filtered} documents below threshold");
    }
}
