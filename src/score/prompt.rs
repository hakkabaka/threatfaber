use std::path::PathBuf;

const SYSTEM_PROMPT: &str = "\
You are a security engineer specializing in threat modeling. \
Evaluate content strictly for relevance to system architecture: \
components, data flows, trust boundaries, protocols, auth mechanisms, \
and deployment topology. \
Score 0-100. Architecture diagrams, schemas, sequence diagrams, network maps, \
trust-boundary views, and deployment views score high. \
Narrative business text, policy prose, UI mockups, cover pages, and marketing \
content score low. \
When scoring images, judge the visual content of each image on its own rather \
than copying relevance from the document text. \
Respond ONLY with valid JSON, no markdown, no explanation.";

const CLEANING_SYSTEM_PROMPT: &str = "\
You are a security engineer cleaning source documentation for threat modeling. \
Keep only content directly useful for system architecture understanding: \
components, services, data stores, external dependencies, protocols, APIs, \
authentication, authorization, deployment topology, network boundaries, trust \
boundaries, secrets handling, data flows, and security-relevant operational \
details. \
Remove business context, project management notes, meeting content, generic \
policies, legal text, marketing language, boilerplate, repeated headers, and \
other prose that does not help reconstruct the system architecture. \
Preserve the surviving content as concise markdown. \
If nothing is relevant, return exactly NONE. \
Respond only with markdown text, no explanation.";

const MAX_MARKDOWN_LEN: usize = 12_000;

pub fn system_prompt() -> &'static str {
    SYSTEM_PROMPT
}

pub fn cleaning_system_prompt() -> &'static str {
    CLEANING_SYSTEM_PROMPT
}

pub fn build_cleaning_prompt(markdown: &str) -> String {
    let trimmed_markdown = trim_markdown(markdown, MAX_MARKDOWN_LEN);

    format!(
        "Rewrite the following extracted document into a cleaned markdown file \
that keeps only system-architecture-relevant information.\n\n\
Keep:\n\
- system components and responsibilities\n\
- data flows and integrations\n\
- APIs, protocols, and interfaces\n\
- authn/authz details\n\
- deployment, hosting, network, and trust-boundary information\n\
- service-to-service communication\n\
- security-relevant operational constraints\n\n\
Remove:\n\
- introductions, summaries, and business context\n\
- meeting notes and action items\n\
- compliance/policy prose without architecture detail\n\
- generic UI/product descriptions\n\
- duplicate or low-signal text\n\n\
Rules:\n\
- Preserve important technical facts.\n\
- Keep the output concise and structured.\n\
- Output markdown only.\n\
- If there is no architecture-relevant information, return exactly NONE.\n\n\
=== SOURCE DOCUMENT ===\n{trimmed_markdown}"
    )
}

pub fn build_prompt(markdown: &str, image_paths: &[PathBuf]) -> String {
    let trimmed_markdown = trim_markdown(markdown, MAX_MARKDOWN_LEN);
    let image_list = image_list(image_paths);

    format!(
        "Score the document text and each image for relevance to system \
architecture and threat modeling (0-100).\n\n\
Use higher scores only when the content describes system components, \
interfaces, trust boundaries, deployments, protocols, authentication, \
authorization, or data flows. If an image is a UI screenshot, branding, \
logo, decorative asset, or mostly prose/text, score it low.\n\n\
Image-specific rules:\n\
- Treat attached images as full rendered document pages, not cropped assets.\n\
- Score an image high only if the page visually shows a diagram or structured \
technical view such as component boxes, arrows, sequence flows, network \
segments, data flow diagrams, deployment topology, trust boundaries, tables of \
services/interfaces, or architecture schemas.\n\
- Score an image low if the page is mostly paragraphs, headings, policy text, \
meeting notes, screenshots of UI, or general documentation without a visible \
system diagram.\n\
- Do not give a high image score just because the document text is relevant; \
the image itself must visually contain architecture-relevant structure.\n\n\
=== DOCUMENT TEXT ===\n{trimmed_markdown}\n\n\
=== ATTACHED IMAGES (in order) ===\n{image_list}\n\n\
Return ONLY this JSON structure:\n\
{{\"text_score\":<0-100>,\"images\":[{{\"filename\":\"<name>\",\"score\":<0-100>}},...]}}"
    )
}

fn image_list(image_paths: &[PathBuf]) -> String {
    if image_paths.is_empty() {
        return "No images attached.".to_string();
    }

    image_paths
        .iter()
        .enumerate()
        .map(|(i, path)| {
            format!(
                "  {}: {}",
                i + 1,
                path.file_name().unwrap_or_default().to_string_lossy()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn trim_markdown(markdown: &str, max_len: usize) -> &str {
    if markdown.len() <= max_len {
        return markdown;
    }

    let mut end = max_len;
    while !markdown.is_char_boundary(end) {
        end -= 1;
    }

    &markdown[..end]
}
