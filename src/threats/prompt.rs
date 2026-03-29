const SYSTEM_PROMPT: &str = "\
You are a senior security engineer producing a threat model from a structured system model. \
Use the provided system model as the primary source of truth. \
Generate a practical markdown report focused on real attack surface, likely abuse paths, and concrete mitigations. \
Do not invent implementation details that are not implied by the system model. \
If information is missing, call it out as an unknown or assumption.";

pub fn system_prompt() -> &'static str {
    SYSTEM_PROMPT
}

pub fn build_prompt(system_model_json: &str) -> String {
    format!(
        "Generate a markdown threat model for the service described by the SystemModel JSON below.\n\n\
The report must include these sections:\n\
1. Scope and assumptions\n\
2. Assets to protect\n\
3. Architecture summary\n\
4. Trust boundaries\n\
5. Entry points and attack surface\n\
6. Threats and mitigations\n\
7. Open questions / unknowns\n\
\n\
For section 6:\n\
- organize threats by component, data flow, or boundary when helpful\n\
- use STRIDE-style reasoning, but keep the output readable markdown, not JSON\n\
- include concrete mitigations, not generic platitudes\n\
- call out missing controls inferred from the model\n\n\
Return markdown only.\n\n\
=== SYSTEM MODEL JSON ===\n{system_model_json}"
    )
}
