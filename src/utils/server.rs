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

pub static CONFIG: std::sync::LazyLock<Config> = std::sync::LazyLock::new(|| {
    let config_str = std::fs::read_to_string("scout.toml").expect("Failed to read config file");
    let config = toml::from_str(&config_str).expect("Failed to parse config file");
    config
});

/// A map of code system URL to a map of code to display string.
type CodeMaps = HashMap<String, HashMap<String, String>>;

pub static CODE_MAPS: std::sync::LazyLock<CodeMaps> = std::sync::LazyLock::new(|| {
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

    // Load all code systems from the codesystems directory.
    let mut code_maps = HashMap::new();
    for entry in std::fs::read_dir("codesystems").expect("Failed to read codesystems directory") {
        let entry = entry.expect("Failed to read codesystems directory entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_content =
                std::fs::read_to_string(&path).expect("Failed to read code system file");
            let code_system: CodeSystem =
                serde_json::from_str(&file_content).expect("Failed to parse code system file");
            let mut code_map = HashMap::new();
            for concept in code_system.concept {
                code_map.insert(concept.code, concept.display);
            }
            code_maps.insert(code_system.url, code_map);
        }
    }
    tracing::info!("Loaded {} code maps", code_maps.len());
    code_maps
});
