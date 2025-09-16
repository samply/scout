#![cfg(feature = "server")]

use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub fhir_base_url: String,
    pub fhir_username: Option<String>,
    pub fhir_password: Option<String>,
    #[serde(default)]
    pub accept_invalid_certs: bool,
}

static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

/// Load the configuration from scout.toml. Called once on server startup.
pub fn load_config() -> anyhow::Result<()> {
    let config_str = std::fs::read_to_string("scout.toml")?;
    let config = toml::from_str(&config_str)?;
    CONFIG.set(config).expect("Config should only be set once");
    Ok(())
}

pub fn config() -> &'static Config {
    CONFIG.get().expect("Config should be loaded before use")
}

/// A map of code system URL to a map of code to display string.
type CodeMaps = HashMap<String, HashMap<String, String>>;

static CODE_MAPS: std::sync::OnceLock<CodeMaps> = std::sync::OnceLock::new();

/// http://hl7.org/fhir/StructureDefinition/CodeSystem
#[derive(Debug, serde::Deserialize)]
struct CodeSystem {
    url: String,
    concept: Vec<CodeSystemConcept>,
}

#[derive(Debug, serde::Deserialize)]
struct CodeSystemConcept {
    code: String,
    display: String,
}

/// Load all code systems from the codesystems directory. Called once on server startup.
pub fn load_code_maps() -> anyhow::Result<()> {
    let mut code_maps = HashMap::new();
    for entry in std::fs::read_dir("codesystems")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_content = std::fs::read_to_string(&path)?;
            let code_system: CodeSystem = serde_json::from_str(&file_content)?;
            let mut code_map = HashMap::new();
            for concept in code_system.concept {
                code_map.insert(concept.code, concept.display);
            }
            code_maps.insert(code_system.url, code_map);
        }
    }
    CODE_MAPS
        .set(code_maps)
        .expect("Code maps should only be set once");
    tracing::info!("Loaded {} code maps", CODE_MAPS.get().unwrap().len());
    Ok(())
}

pub fn code_maps() -> &'static CodeMaps {
    CODE_MAPS
        .get()
        .expect("Code maps should be loaded before use")
}
