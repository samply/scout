use serde::{Deserialize, Serialize};
use std::fmt;

/// http://hl7.org/fhir/StructureDefinition/HumanName
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanName {
    text: Option<String>,
    family: Option<String>,
    given: Option<Vec<String>>,
    prefix: Option<Vec<String>>,
    suffix: Option<Vec<String>>,
}

impl fmt::Display for HumanName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref text) = self.text {
            write!(f, "{}", text)
        } else {
            write!(
                f,
                "{}",
                self.prefix
                    .iter()
                    .flatten()
                    .chain(self.given.iter().flatten())
                    .chain(self.family.iter())
                    .chain(self.suffix.iter().flatten())
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        }
    }
}

/// https://www.medizininformatik-initiative.de/fhir/core/modul-person/StructureDefinition/Patient
/// https://www.medizininformatik-initiative.de/fhir/core/modul-person/StructureDefinition/PatientPseudonymisiert
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    pub id: String,
    pub name: Option<Vec<HumanName>>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    pub deceased_boolean: Option<bool>,
    pub address: Option<Vec<Address>>,
}

impl Patient {
    pub fn name(&self) -> String {
        self.name
            .iter()
            .flatten()
            .map(|name| name.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn gender(&self) -> String {
        self.gender.clone().unwrap_or_default()
    }

    pub fn birth_date(&self) -> String {
        self.birth_date.clone().unwrap_or_default()
    }

    pub fn deceased(&self) -> String {
        self.deceased_boolean
            .map(|deceased| deceased.to_string())
            .unwrap_or_default()
    }

    pub fn address(&self) -> String {
        self.address
            .iter()
            .flatten()
            .map(|address| address.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// http://hl7.org/fhir/StructureDefinition/Address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    text: Option<String>,
    line: Option<Vec<String>>,
    city: Option<String>,
    district: Option<String>,
    state: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref text) = self.text {
            write!(f, "{}", text)
        } else {
            write!(
                f,
                "{}",
                self.line
                    .iter()
                    .flatten()
                    .chain(self.city.iter())
                    .chain(self.district.iter())
                    .chain(self.state.iter())
                    .chain(self.postal_code.iter())
                    .chain(self.country.iter())
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}
