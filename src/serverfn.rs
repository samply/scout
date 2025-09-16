use crate::fhir;
use crate::server;
use dioxus::prelude::*;

pub trait RequestBuilderExt {
    fn with_auth(self) -> Self;
}

#[cfg(feature = "server")]
impl crate::serverfn::RequestBuilderExt for reqwest::RequestBuilder {
    fn with_auth(self) -> Self {
        if let Some(fhir_username) = &server::config().fhir_username {
            self.basic_auth(fhir_username, server::config().fhir_password.as_deref())
        } else {
            self
        }
    }
}

#[server]
pub async fn get_patients() -> Result<Vec<fhir::Patient>, ServerFnError> {
    let url = format!("{}/Patient?_count=10000", server::config().fhir_base_url);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(server::config().accept_invalid_certs)
        .build()?;
    let bundle = client
        .get(&url)
        .with_auth()
        .send()
        .await?
        .error_for_status()?
        .json::<fhir::FhirBundle<fhir::Patient>>()
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
) -> Result<(fhir::Patient, fhir::MixedBundle), ServerFnError> {
    let url = format!(
        "{}/Patient/{}/$everything",
        server::config().fhir_base_url,
        id
    );
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(server::config().accept_invalid_certs)
        .build()?;
    let bundle = client
        .get(&url)
        .with_auth()
        .send()
        .await?
        .error_for_status()?
        .json::<fhir::MixedBundle>()
        .await?;

    let patient = bundle
        .entry
        .iter()
        .find_map(|entry| {
            if let fhir::Resource::Patient(patient) = &entry.resource {
                Some(patient.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| ServerFnError::new("No patient found"))?;

    Ok((patient, bundle))
}
