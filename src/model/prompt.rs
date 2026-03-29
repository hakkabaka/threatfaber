const SYSTEM_PROMPT: &str = "\
You are a security engineer extracting a structured system model from technical documentation. \
Build a precise JSON representation that can be used later for threat modeling. \
Only include facts that are explicit in the source text. \
If information is missing, leave fields empty and record the gap in unknowns. \
Respond only with valid JSON.";

pub fn system_prompt() -> &'static str {
    SYSTEM_PROMPT
}

pub fn build_prompt(markdown: &str) -> String {
    format!(
        "Extract a SystemModel JSON object from the architecture-relevant markdown below.\n\n\
Focus on:\n\
- components, services, clients, data stores, and external systems\n\
- data flows and communication paths\n\
- trust or deployment boundaries\n\
- authentication and authorization mechanisms\n\
- sensitive data classes and where they move or are stored\n\
- external integrations\n\
- assumptions and unknowns relevant to threat modeling\n\n\
Rules:\n\
- Use concise factual strings.\n\
- Do not invent missing details.\n\
- If the document contains little useful architecture information, return mostly empty arrays and explain gaps in unknowns.\n\n\
Return exactly this JSON shape:\n\
{{\n\
  \"summary\": \"string\",\n\
  \"components\": [{{\"name\":\"string\",\"kind\":\"string\",\"description\":\"string\",\"responsibilities\":[\"string\"],\"technologies\":[\"string\"],\"data_handled\":[\"string\"],\"authn\":[\"string\"],\"authz\":[\"string\"]}}],\n\
  \"data_flows\": [{{\"source\":\"string\",\"destination\":\"string\",\"description\":\"string\",\"protocol\":\"string\",\"data\":[\"string\"],\"authentication\":\"string\",\"trust_boundary_crossing\":false}}],\n\
  \"trust_boundaries\": [{{\"name\":\"string\",\"description\":\"string\",\"components\":[\"string\"]}}],\n\
  \"auth_mechanisms\": [{{\"name\":\"string\",\"applies_to\":[\"string\"],\"description\":\"string\"}}],\n\
  \"sensitive_data\": [{{\"name\":\"string\",\"description\":\"string\",\"stored_in\":[\"string\"],\"transmitted_via\":[\"string\"]}}],\n\
  \"external_integrations\": [{{\"name\":\"string\",\"description\":\"string\",\"direction\":\"string\",\"protocol\":\"string\"}}],\n\
  \"assumptions\": [\"string\"],\n\
  \"unknowns\": [\"string\"]\n\
}}\n\n\
=== ARCHITECTURE MARKDOWN ===\n{markdown}"
    )
}
