# threatfaber

`threatfaber` is a Rust CLI that turns architecture PDFs into a service-level system model and a markdown threat model.

It is designed for security review workflows where the source material is noisy:
- architecture decks
- design docs exported to PDF
- mixed text + diagrams
- bloated internal documentation with a lot of irrelevant prose

The current pipeline:
1. Extract text from PDFs
2. Render each PDF page to PNG
3. Clean each document with an LLM to keep only architecture-relevant content
4. Score each document and page for architectural relevance
5. Build one merged `system_model.json` from all related files
6. Generate one final `threat_model.md` from that merged system model

## Why This Exists

Threat modeling is weak when it starts from isolated text chunks or raw document dumps.

This tool uses a workflow-first approach:
- clean source docs first
- extract a structured system view second
- generate the threat model from the structured system view

That gives you a reusable artifact chain instead of one opaque LLM response.

## Features

- PDF text extraction with `unpdf`
- Full-page PDF rendering with `pdfium-render` so vector diagrams are preserved
- LLM-based cleaning of source docs into architecture-focused markdown
- Multimodal relevance scoring over cleaned text plus rendered page images
- Service-level `system_model.json` generated from all cleaned/raw docs in one run
- Final markdown `threat_model.md` generated from the merged system model
- Different OpenAI models per stage, with env-var overrides

## Current Pipeline

For every input PDF:
- `content.md` is extracted from the PDF text
- `page-*.png` images are rendered from the PDF pages
- `cleaned_content.md` is produced by the cleaner model
- relevance scores are generated for cleaned text and rendered pages

For the whole service:
- all cleaned docs are merged
- one `system_model.json` is extracted
- one `threat_model.md` is generated

## Project Layout

```text
src/
  extract/
    pdf.rs
    pdfium.rs
  score/
    cleaner.rs
    images.rs
    prompt.rs
    scorer.rs
    types.rs
  model/
    extractor.rs
    prompt.rs
    types.rs
  threats/
    generator.rs
    prompt.rs
  main.rs
```

## Requirements

- Rust toolchain
- OpenAI API key in environment
- PDFium shared library available at runtime

Required environment:

```bash
export OPENAI_API_KEY=...
```

## PDFium Runtime

Threatfaber renders full PDF pages to PNG so vector diagrams and schemes are captured even when they are not embedded as normal image resources.

The app looks for the PDFium shared library in this order:
1. `PDFIUM_LIB_PATH`
2. Next to the `threatfaber` executable
3. Current working directory
4. System library search path

Platform-specific library names:
- Linux: `libpdfium.so`
- macOS: `libpdfium.dylib`
- Windows: `pdfium.dll`

Recommended release layout:

```text
dist/
  threatfaber
  libpdfium.so
```

If you package the binary for distribution, ship the matching PDFium library beside the executable.

## Usage

Run against a directory containing PDF files:

```bash
./threatfaber --input ./sample-pdfs
```

Custom output directory:

```bash
./threatfaber --input ./sample-pdfs --output ./result
```

Adjust the relevance threshold:

```bash
./threatfaber --input ./sample-pdfs --min-score 70
```

## Output

Example output tree:

```text
result/
  document_scores.json
  relevant_documents.json
  filtered_out_documents.json
  system_model.json
  threat_model.md
  extract/
    <doc_id>/
      content.md
      cleaned_content.md
      page-001.png
      page-002.png
      ...
```

### Per-document artifacts

- `content.md`: raw extracted markdown from the PDF
- `cleaned_content.md`: cleaned architecture-focused markdown
- `page-*.png`: rendered PDF pages used for multimodal scoring

### Service-level artifacts

- `system_model.json`: merged model built from all docs related to the service
- `threat_model.md`: final markdown threat model generated from the merged system model

### Score files

- `document_scores.json`: all scored documents
- `relevant_documents.json`: documents above threshold
- `filtered_out_documents.json`: documents below threshold

## OpenAI Model Split

The default stage-to-model mapping is:

- cleaner: `gpt-5-nano`
- scorer: `gpt-5-mini`
- system model extraction: `gpt-5.2`
- threat model generation: `gpt-5.2`

These defaults are chosen to keep high-volume preprocessing cheap while using a frontier model for the two reasoning-heavy stages.

Override any stage with env vars:

```bash
export THREATFABER_CLEANER_MODEL=gpt-5-nano
export THREATFABER_SCORER_MODEL=gpt-5-mini
export THREATFABER_SYSTEM_MODEL=gpt-5.2
export THREATFABER_THREAT_MODEL=gpt-5.2
```

## Design Notes

- Cleaning is intentionally separate from system-model extraction.
  This keeps the architecture-relevant text visible on disk and debuggable.
- The system model is built from all files in the run, not per document.
  That better matches real services where architecture details are spread across multiple files.
- Threat generation runs from `system_model.json`, not raw docs.
  That keeps the final reasoning stage focused and reproducible.

## Limitations

- Input is currently PDF-only
- The merged service model can become large if many source documents are included
- Threat quality depends on source quality and the cleaner/model prompts
- No RAG or citation-grounding step yet
- No merge conflict resolution across contradictory docs yet

## Roadmap

- Better service-level document grouping
- Stronger structured extraction with field confidence/source tracking
- Threat citations back to cleaned docs
- Support for Markdown and Confluence exports
- Incremental caching by document hash
- Report export formats beyond markdown

## Development

Build check:

```bash
cargo check
```

The project currently targets:
- Rust
- `rig-core` for OpenAI calls
- `pdfium-render` for page rendering
- `unpdf` for text extraction