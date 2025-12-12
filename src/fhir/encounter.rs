use crate::fhir::code::{Chip, CodeableConcept, Coding, Identifier, Period, Reference};
use serde::{Deserialize, Serialize};

/// https://www.medizininformatik-initiative.de/fhir/core/modul-fall/StructureDefinition/KontaktGesundheitseinrichtung
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub id: String,
    pub identifier: Option<Vec<Identifier>>,
    pub status: String,
    pub class: Coding,
    pub r#type: Option<Vec<CodeableConcept>>,
    pub service_type: Option<CodeableConcept>,
    pub period: Option<Period>,
    pub service_provider: Option<Reference>,
}

impl Encounter {
    pub fn visit_number(&self) -> String {
        self.identifier
            .iter()
            .flatten()
            .find_map(|identifier| {
                identifier
                    .r#type
                    .as_ref()?
                    .coding
                    .iter()
                    .flatten()
                    .any(|c| c.code == Some("VN".into()))
                    .then(|| identifier.value.clone())?
            })
            .unwrap_or_default()
    }

    /// http://fhir.de/ValueSet/EncounterStatusDe
    #[rustfmt::skip]
    pub fn status_chip(&self) -> Option<Chip> {
        match self.status.as_str() {
            "planned" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Planned", "The Encounter has not yet started.")),
            "in-progress" => Some(Chip::new("bg-yellow-100 border-yellow-500", "In Progress", "The Encounter has begun and the patient is present / the practitioner and the patient are meeting.")),
            "onleave" => Some(Chip::new("bg-gray-100 border-gray-500", "On Leave", "The Encounter has begun, but the patient is temporarily on leave.")),
            "finished" => Some(Chip::new("bg-green-100 border-green-500", "Finished", "The Encounter has ended.")),
            "cancelled" => Some(Chip::new("bg-red-100 border-red-500", "Cancelled", "The Encounter has ended before it has begun.")),
            "entered-in-error" => Some(Chip::new("bg-purple-100 border-purple-500", "Entered in Error", "This instance should not have been part of this patient's medical record.")),
            "unknown" => Some(Chip::new("bg-gray-100 border-gray-500", "Unknown", "The encounter status is unknown. Note that \"unknown\" is a value of last resort and every attempt should be made to provide a meaningful value other than \"unknown\".")),
            _ => None,
        }
    }

    pub fn class(&self) -> String {
        self.class.display.clone().unwrap_or_default()
    }

    /// http://fhir.de/CodeSystem/Kontaktebene
    pub fn encounter_level(&self) -> String {
        self.r#type
            .iter()
            .flatten()
            .find(|r#type| {
                r#type.coding.iter().flatten().any(|coding| {
                    coding.system == Some("http://fhir.de/CodeSystem/Kontaktebene".into())
                })
            })
            .map(|r#type| r#type.to_string())
            .unwrap_or_default()
    }

    /// http://fhir.de/CodeSystem/kontaktart-de
    pub fn encounter_type(&self) -> String {
        self.r#type
            .iter()
            .flatten()
            .find(|r#type| {
                r#type.coding.iter().flatten().any(|coding| {
                    coding.system == Some("http://fhir.de/CodeSystem/kontaktart-de".into())
                })
            })
            .map(|r#type| r#type.to_string())
            .unwrap_or_default()
    }

    pub fn service_type(&self) -> String {
        self.service_type
            .as_ref()
            .map(|service_type| service_type.to_string())
            .unwrap_or_default()
    }

    pub fn service_provider(&self) -> String {
        self.service_provider
            .as_ref()
            .and_then(|service_provider| service_provider.identifier.as_ref()?.value.clone())
            .unwrap_or_default()
    }

    pub fn timeline_timestamp(&self) -> Option<jiff::Timestamp> {
        self.period.as_ref().and_then(|period| period.start)
    }
}
