//! This module contains the data structures for the FHIR resources used in the application.

use serde::{Deserialize, Serialize};
use std::fmt;

pub fn format_time(timestamp: jiff::Timestamp) -> String {
    let zoned = timestamp.to_zoned(jiff::tz::TimeZone::system());
    // Jan 08, 2020, 07:00 CET
    zoned.strftime("%b %d, %Y, %H:%M %Z").to_string()
}

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

/// https://www.medizininformatik-initiative.de/fhir/core/modul-person/StructureDefinition/Patient
/// https://www.medizininformatik-initiative.de/fhir/core/modul-person/StructureDefinition/PatientPseudonymisiert
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    pub id: Option<String>,
    pub name: Option<Vec<HumanName>>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    pub deceased_boolean: Option<bool>,
    pub address: Option<Vec<Address>>,
}

impl Patient {
    pub fn id(&self) -> String {
        self.id.clone().unwrap_or_default()
    }

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

/// http://hl7.org/fhir/StructureDefinition/Coding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coding {
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

impl fmt::Display for Coding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref display) = self.display {
            write!(f, "{}", display)
        } else {
            write!(f, "{}", self.code.clone().unwrap_or_default())
        }
    }
}

/// http://hl7.org/fhir/StructureDefinition/CodeableConcept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeableConcept {
    pub coding: Option<Vec<Coding>>,
    pub text: Option<String>,
}

impl fmt::Display for CodeableConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref text) = self.text {
            write!(f, "{}", text)
        } else {
            write!(
                f,
                "{}",
                self.coding
                    .iter()
                    .flatten()
                    .map(|coding| coding.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

impl CodeableConcept {
    pub fn code_in_system(&self, system: &str) -> Option<String> {
        self.coding
            .as_ref()?
            .iter()
            .find(|coding| coding.system.as_deref() == Some(system))
            .and_then(|coding| coding.code.clone())
    }
}

/// http://hl7.org/fhir/StructureDefinition/Period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    pub start: Option<jiff::Timestamp>,
    pub end: Option<jiff::Timestamp>,
}

/// http://hl7.org/fhir/StructureDefinition/Identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub r#type: Option<CodeableConcept>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub reference: Option<String>,
    pub identifier: Option<Identifier>,
}

/// http://hl7.org/fhir/StructureDefinition/Annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub time: Option<jiff::Timestamp>,
    pub text: String,
}

/// https://www.medizininformatik-initiative.de/fhir/core/modul-fall/StructureDefinition/KontaktGesundheitseinrichtung
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub id: Option<String>,
    pub identifier: Option<Vec<Identifier>>,
    pub status: String,
    pub class: Coding,
    pub r#type: Option<Vec<CodeableConcept>>,
    pub service_type: Option<CodeableConcept>,
    pub period: Option<Period>,
    pub service_provider: Option<Reference>,
}

impl Encounter {
    pub fn id(&self) -> String {
        self.id.clone().unwrap_or_default()
    }

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
}

impl TimelineEvent for Encounter {
    fn timestamp(&self) -> Option<jiff::Timestamp> {
        self.period.as_ref().and_then(|period| period.start)
    }
}

/// https://www.medizininformatik-initiative.de/fhir/core/modul-diagnose/StructureDefinition/Diagnose
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub id: Option<String>,
    pub clinical_status: Option<CodeableConcept>,
    pub verification_status: Option<CodeableConcept>,
    pub code: CodeableConcept,
    pub body_site: Option<Vec<CodeableConcept>>,
    pub onset_period: Option<Period>,
    pub onset_date_time: Option<jiff::Timestamp>,
    pub recorded_date: jiff::Timestamp,
    pub note: Option<Vec<Annotation>>,
}

impl Condition {
    pub fn id(&self) -> String {
        self.id.clone().unwrap_or_default()
    }

    /// http://hl7.org/fhir/ValueSet/condition-clinical
    pub fn clinical_status_chip(&self) -> Option<Chip> {
        match self.clinical_status.as_ref()?.code_in_system("http://terminology.hl7.org/CodeSystem/condition-clinical")?.as_str() {
            "active" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Active", "The subject is currently experiencing the symptoms of the condition or there is evidence of the condition.")),
            "recurrence" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Recurrence", "The subject is experiencing a re-occurrence or repeating of a previously resolved condition, e.g. urinary tract infection, pancreatitis, cholangitis, conjunctivitis.")),
            "relapse" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Relapse", "The subject is experiencing a return of a condition, or signs and symptoms after a period of improvement or remission, e.g. relapse of cancer, multiple sclerosis, rheumatoid arthritis, systemic lupus erythematosus, bipolar disorder, [psychotic relapse of] schizophrenia, etc.")),
            "inactive" => Some(Chip::new("bg-gray-100 border-gray-500", "Inactive", "The subject is no longer experiencing the symptoms of the condition or there is no longer evidence of the condition.")),
            "remission" => Some(Chip::new("bg-green-100 border-green-500", "Remission", "The subject is no longer experiencing the symptoms of the condition, but there is a risk of the symptoms returning.")),
            "resolved" => Some(Chip::new("bg-green-100 border-green-500", "Resolved", "The subject is no longer experiencing the symptoms of the condition and there is a negligible perceived risk of the symptoms returning.")),
            _ => None,
        }
    }

    pub fn clinical_status(&self) -> String {
        self.clinical_status
            .as_ref()
            .map(|status| status.to_string())
            .unwrap_or_default()
    }

    /// http://hl7.org/fhir/ValueSet/condition-ver-status
    pub fn verification_status_chip(&self) -> Option<Chip> {
        match self.verification_status.as_ref()?.code_in_system("http://terminology.hl7.org/CodeSystem/condition-ver-status")?.as_str() {
            "unconfirmed" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Unconfirmed", "There is not sufficient diagnostic and/or clinical evidence to treat this as a confirmed condition.")),
            "provisional" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Provisional", "This is a tentative diagnosis - still a candidate that is under consideration.")),
            "differential" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Differential", "One of a set of potential (and typically mutually exclusive) diagnoses asserted to further guide the diagnostic process and preliminary treatment.")),
            "confirmed" => Some(Chip::new("bg-green-100 border-green-500", "Confirmed", "There is sufficient diagnostic and/or clinical evidence to treat this as a confirmed condition.")),
            "refuted" => Some(Chip::new("bg-red-100 border-red-500", "Refuted", "This condition has been ruled out by diagnostic and clinical evidence.")),
            "entered-in-error" => Some(Chip::new("bg-purple-100 border-purple-500", "Entered in Error", "The statement was entered in error and is not valid.")),
            _ => None,
        }
    }

    pub fn verification_status(&self) -> String {
        self.verification_status
            .as_ref()
            .map(|status| status.to_string())
            .unwrap_or_default()
    }

    pub fn code(&self) -> String {
        self.code.to_string()
    }

    pub fn body_site(&self) -> String {
        self.body_site
            .iter()
            .flatten()
            .map(|site| site.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn onset_start(&self) -> String {
        self.onset_period
            .as_ref()
            .and_then(|period| period.start)
            .or(self.onset_date_time)
            .map(|timestamp| format_time(timestamp))
            .unwrap_or_default()
    }

    pub fn note(&self) -> String {
        self.note
            .iter()
            .flatten()
            .map(|note| note.text.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl TimelineEvent for Condition {
    fn timestamp(&self) -> Option<jiff::Timestamp> {
        Some(self.recorded_date)
    }
}

/// https://www.medizininformatik-initiative.de/fhir/core/modul-prozedur/StructureDefinition/Procedure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Procedure {
    pub id: Option<String>,
    pub status: String,
    pub category: Option<CodeableConcept>,
    pub code: CodeableConcept,
    pub performed_date_time: Option<jiff::Timestamp>,
    pub performed_period: Option<Period>,
    pub body_site: Option<Vec<CodeableConcept>>,
    pub note: Option<Vec<Annotation>>,
}

impl Procedure {
    pub fn id(&self) -> String {
        self.id.clone().unwrap_or_default()
    }

    /// http://hl7.org/fhir/ValueSet/event-status
    pub fn status_chip(&self) -> Option<Chip> {
        match self.status.as_str() {
            "preparation" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Preparation", "The core event has not started yet, but some staging activities have begun (e.g. surgical suite preparation). Preparation stages may be tracked for billing purposes.")),
            "in-progress" => Some(Chip::new("bg-yellow-100 border-yellow-500", "In Progress", "The event is currently occurring.")),
            "not-done" => Some(Chip::new("bg-purple-100 border-purple-500", "Not Done", "The event was terminated prior to any activity beyond preparation. I.e. The 'main' activity has not yet begun. The boundary between preparatory and the 'main' activity is context-specific.")),
            "on-hold" => Some(Chip::new("bg-yellow-100 border-yellow-500", "On Hold", "The event has been temporarily stopped but is expected to resume in the future.")),
            "stopped" => Some(Chip::new("bg-purple-100 border-purple-500", "Stopped", "The event was terminated prior to the full completion of the intended activity but after at least some of the 'main' activity (beyond preparation) has occurred.")),
            "completed" => Some(Chip::new("bg-green-100 border-green-500", "Completed", "The event has now concluded.")),
            "entered-in-error" => Some(Chip::new("bg-purple-100 border-purple-500", "Entered in Error", "This electronic record should never have existed, though it is possible that real-world decisions were based on it. (If real-world activity has occurred, the status should be \"stopped\" rather than \"entered-in-error\".)")),
            "unknown" => Some(Chip::new("bg-gray-100 border-gray-500", "Unknown", "The authoring/source system does not know which of the status values currently applies for this event. Note: This concept is not to be used for \"other\" - one of the listed statuses is presumed to apply, but the authoring/source system does not know which.")),
            _ => None,
        }
    }

    pub fn category(&self) -> String {
        self.category
            .as_ref()
            .map(|category| category.to_string())
            .unwrap_or_default()
    }

    pub fn code(&self) -> String {
        self.code.to_string()
    }

    pub fn body_site(&self) -> String {
        self.body_site
            .iter()
            .flatten()
            .map(|site| site.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn note(&self) -> String {
        self.note
            .iter()
            .flatten()
            .map(|note| note.text.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl TimelineEvent for Procedure {
    fn timestamp(&self) -> Option<jiff::Timestamp> {
        self.performed_period
            .as_ref()
            .and_then(|period| period.start)
            .or(self.performed_date_time)
    }
}

pub trait TimelineEvent {
    /// Returns the timestamp that is used to sort events in the timeline. If
    /// `None` is returned, the event will not be included in the timeline.
    fn timestamp(&self) -> Option<jiff::Timestamp>;

    fn formatted_timestamp(&self) -> String {
        self.timestamp()
            .map(format_time)
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirEntry<T> {
    pub resource: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirBundle<T> {
    pub entry: Vec<FhirEntry<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "resourceType")]
pub enum Resource {
    Patient(Patient),
    Encounter(Encounter),
    Condition(Condition),
    Procedure(Procedure),
    #[serde(other)]
    Unknown,
}

impl Resource {
    pub fn timeline_event(&self) -> Option<&dyn TimelineEvent> {
        match self {
            Resource::Encounter(encounter) => Some(encounter),
            Resource::Condition(condition) => Some(condition),
            Resource::Procedure(procedure) => Some(procedure),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedEntry {
    pub resource: Resource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedBundle {
    pub entry: Vec<MixedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chip {
    pub class: String,
    pub text: String,
    pub hover_text: String,
}

impl Chip {
    pub fn new(class: &str, text: &str, hover_text: &str) -> Self {
        Self {
            class: class.to_string(),
            text: text.to_string(),
            hover_text: hover_text.to_string(),
        }
    }
}
