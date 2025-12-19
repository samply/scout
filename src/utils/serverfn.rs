use crate::fhir::{condition, patient, resources};
#[cfg(feature = "server")]
use crate::utils::config;
use dioxus::prelude::server;
use dioxus::prelude::*;

pub trait RequestBuilderExt {
    fn with_auth(self) -> Self;
}

#[cfg(feature = "server")]
impl crate::utils::serverfn::RequestBuilderExt for reqwest::RequestBuilder {
    fn with_auth(self) -> Self {
        if let Some(fhir_username) = &config::CONFIG.fhir_username {
            self.basic_auth(fhir_username, config::CONFIG.fhir_password.as_deref())
        } else {
            self
        }
    }
}

#[server]
pub async fn get_patients() -> Result<Vec<patient::Patient>, ServerFnError> {
    let url = format!("{}/Patient?_count=10000", config::CONFIG.fhir_base_url);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(config::CONFIG.accept_invalid_certs)
        .build()?;
    let bundle = client
        .get(&url)
        .with_auth()
        .send()
        .await?
        .error_for_status()?
        .json::<resources::FhirBundle<patient::Patient>>()
        .await?;
    Ok(bundle
        .entry
        .into_iter()
        .map(|entry| entry.resource)
        .collect())
}

#[server]
pub async fn get_conditions() -> Result<Vec<condition::Condition>, ServerFnError> {
    let url = format!("{}/Condition?_count=10000", config::CONFIG.fhir_base_url);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(config::CONFIG.accept_invalid_certs)
        .build()?;
    let bundle = client
        .get(&url)
        .with_auth()
        .send()
        .await?
        .error_for_status()?
        .json::<resources::FhirBundle<condition::Condition>>()
        .await?;
    Ok(bundle
        .entry
        .into_iter()
        .map(|entry| entry.resource)
        .collect())
}

/// Get a patient and their related resources.
#[server]
pub async fn get_patient_details(
    id: String,
) -> Result<(patient::Patient, resources::MixedBundle), ServerFnError> {
    let url = format!(
        "{}/Patient/{}/$everything",
        config::CONFIG.fhir_base_url,
        id
    );
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(config::CONFIG.accept_invalid_certs)
        .build()?;
    let bundle = client
        .get(&url)
        .with_auth()
        .send()
        .await?
        .error_for_status()?
        .json::<resources::MixedBundle>()
        .await?;

    let patient = bundle
        .entry
        .iter()
        .find_map(|entry| {
            if let resources::Resource::Patient(patient) = &entry.resource {
                Some(patient.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| ServerFnError::new("No patient found"))?;

    Ok((patient, bundle))
}
