use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SystemModel {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub data_flows: Vec<DataFlow>,
    #[serde(default)]
    pub trust_boundaries: Vec<TrustBoundary>,
    #[serde(default)]
    pub auth_mechanisms: Vec<AuthMechanism>,
    #[serde(default)]
    pub sensitive_data: Vec<SensitiveData>,
    #[serde(default)]
    pub external_integrations: Vec<ExternalIntegration>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub unknowns: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub responsibilities: Vec<String>,
    #[serde(default)]
    pub technologies: Vec<String>,
    #[serde(default)]
    pub data_handled: Vec<String>,
    #[serde(default)]
    pub authn: Vec<String>,
    #[serde(default)]
    pub authz: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DataFlow {
    pub source: String,
    pub destination: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub data: Vec<String>,
    #[serde(default)]
    pub authentication: String,
    #[serde(default)]
    pub trust_boundary_crossing: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrustBoundary {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub components: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthMechanism {
    pub name: String,
    #[serde(default)]
    pub applies_to: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SensitiveData {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub stored_in: Vec<String>,
    #[serde(default)]
    pub transmitted_via: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExternalIntegration {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub direction: String,
    #[serde(default)]
    pub protocol: String,
}
