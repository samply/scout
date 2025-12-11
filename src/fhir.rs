//! This module contains the data structures for the FHIR resources used in the application.

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

/// Helper struct for looking up code display names during deserialization. On the server side we
/// deserialize as `RawCoding` and then convert to `Coding`. The `From` implementation handles
/// the lookup in the code maps.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCoding {
    system: Option<String>,
    code: Option<String>,
    display: Option<String>,
    user_selected: Option<bool>,
}

/// http://hl7.org/fhir/StructureDefinition/Coding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "server", serde(from = "RawCoding"))]
pub struct Coding {
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
    pub user_selected: Option<bool>,
}

#[cfg(feature = "server")]
impl From<RawCoding> for Coding {
    fn from(
        RawCoding {
            system,
            code,
            mut display,
            user_selected,
        }: RawCoding,
    ) -> Coding {
        if let (Some(code), Some(system)) = (&code, &system) {
            display = crate::server::CODE_MAPS
                .get(system)
                .and_then(|map| map.get(code))
                .cloned()
                .or(display);
        }
        Coding {
            system,
            code,
            display,
            user_selected,
        }
    }
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// https://www.medizininformatik-initiative.de/fhir/core/modul-diagnose/StructureDefinition/Diagnose
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub id: String,
    pub clinical_status: Option<CodeableConcept>,
    pub verification_status: Option<CodeableConcept>,
    pub code: CodeableConcept,
    pub body_site: Option<Vec<CodeableConcept>>,
    pub subject: Reference,
    pub onset_period: Option<Period>,
    pub onset_date_time: Option<jiff::Timestamp>,
    pub recorded_date: Option<jiff::Timestamp>,
    pub note: Option<Vec<Annotation>>,
}

impl Condition {
    /// http://hl7.org/fhir/ValueSet/condition-clinical
    #[rustfmt::skip]
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
    #[rustfmt::skip]
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

    pub fn subject_patient_id(&self) -> Option<String> {
        self.subject
            .reference
            .as_ref()
            .and_then(|r| r.strip_prefix("Patient/"))
            .map(|id| id.to_string())
    }

    pub fn note(&self) -> String {
        self.note
            .iter()
            .flatten()
            .map(|note| note.text.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn timeline_timestamp(&self) -> Option<jiff::Timestamp> {
        self.recorded_date
    }

    pub fn is_neoplasm(&self) -> bool {
        self.code
            .code_in_system("http://fhir.de/CodeSystem/bfarm/icd-10-gm")
            .is_some_and(|c| "C00".to_string() <= c && c <= "D48.9".to_string())
    }
}

/// https://www.medizininformatik-initiative.de/fhir/core/modul-prozedur/StructureDefinition/Procedure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Procedure {
    pub id: String,
    pub status: String,
    pub category: Option<CodeableConcept>,
    pub code: CodeableConcept,
    pub performed_date_time: Option<jiff::Timestamp>,
    pub performed_period: Option<Period>,
    pub body_site: Option<Vec<CodeableConcept>>,
    pub note: Option<Vec<Annotation>>,
}

impl Procedure {
    /// http://hl7.org/fhir/ValueSet/event-status
    #[rustfmt::skip]
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

    pub fn timeline_timestamp(&self) -> Option<jiff::Timestamp> {
        self.performed_period
            .as_ref()
            .and_then(|period| period.start)
            .or(self.performed_date_time)
    }

    pub fn is_radiation_therapy_or_nuclear_medicine_therapy_or_chemotherapy(&self) -> bool {
        self.code
            .code_in_system("http://fhir.de/CodeSystem/bfarm/ops")
            .is_some_and(|c| "8-520".to_string() <= c && c <= "8-549.x".to_string())
    }
}

/// http://hl7.org/fhir/StructureDefinition/Quantity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    pub value: Option<f64>,
    pub comparator: Option<String>,
    pub unit: Option<String>,
    pub system: Option<String>,
    pub code: Option<String>,
}

/// Quantity where the `comparator` is not used.
pub type SimpleQuantity = Quantity;

impl Quantity {
    pub fn try_to_string(&self) -> Option<String> {
        self.value.map(|value| {
            let value_and_unit = if let Some(unit) = &self.unit {
                format!("{value} {unit}")
            } else {
                value.to_string()
            };
            if let Some(comparator) = &self.comparator {
                format!("{comparator} {value_and_unit}")
            } else {
                value_and_unit
            }
        })
    }
}

/// https://www.medizininformatik-initiative.de/fhir/core/modul-labor/StructureDefinition/ObservationLab
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub id: String,
    pub identifier: Option<Vec<Identifier>>,
    pub status: Option<String>,
    pub category: Option<Vec<CodeableConcept>>,
    pub code: CodeableConcept,
    pub encounter: Option<Reference>,
    pub effective_date_time: Option<jiff::Timestamp>,
    pub issued: Option<jiff::Timestamp>,
    pub value_quantity: Option<Quantity>,
    pub data_absent_reason: Option<CodeableConcept>,
    pub interpretation: Option<Vec<CodeableConcept>>,
    pub note: Option<Vec<Annotation>>,
    pub method: Option<CodeableConcept>,
    pub reference_range: Option<Vec<ObservationReferenceRange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationReferenceRange {
    pub low: Option<SimpleQuantity>,
    pub high: Option<SimpleQuantity>,
    pub r#type: Option<CodeableConcept>,
}

impl Observation {
    pub fn identifier(&self) -> Option<String> {
        self.identifier.as_ref()?.iter()
            .find_map(|id| {
                let is_obi = id.r#type
                    .as_ref()
                    .and_then(|t| t.code_in_system("http://terminology.hl7.org/CodeSystem/v2-0203"))
                    .as_deref() == Some("OBI");

                if is_obi {
                    id.value.clone()
                } else {
                    None
                }
            })
    }
    
    /// http://hl7.org/fhir/ValueSet/observation-status
    #[rustfmt::skip]
    pub fn status_chip(&self) -> Option<Chip> {
        match self.status.as_deref().unwrap_or("") {
            "registered" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Registered", "The existence of the observation is registered, but there is no result yet available.")),
            "preliminary" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Preliminary", "This is an initial or interim observation: data may be incomplete or unverified.")),
            "final" => Some(Chip::new("bg-green-100 border-green-500", "Final", "The observation is complete and there are no further actions needed. Additional information such as 'released', 'signed', etc would be represented using Provenance.")),
            "amended" => Some(Chip::new("bg-purple-100 border-purple-500", "Amended", "Subsequent to being Final, the observation has been modified subsequent. This includes updates/new information and corrections.")),
            "corrected" => Some(Chip::new("bg-purple-100 border-purple-500", "Corrected", "Subsequent to being Final, the observation has been modified to correct an error in the test result.")),
            "cancelled" => Some(Chip::new("bg-red-100 border-red-500", "Cancelled", "The observation is unavailable because the measurement was not started or not completed (also sometimes called 'aborted').")),
            "entered-in-error" => Some(Chip::new("bg-purple-100 border-purple-500", "Entered in Error", "The observation has been withdrawn following previous final release. This electronic record should never have existed, though it is possible that real-world decisions were based on it.")),
            "unknown" => Some(Chip::new("bg-gray-100 border-gray-500", "Unknown", "The authoring/source system does not know which of the status values currently applies for this observation. Note: This concept is not to be used for 'other' - one of the listed statuses is presumed to apply, but the authoring/source system does not know which.")),
            _ => None,
        }
    }

    pub fn category(&self) -> Option<String> {
        match self.category.as_ref() {
            Some(category) => {
                Some(category
                    .iter()
                    .map(|category| category.to_string())
                    .collect::<Vec<_>>()
                    .join(", "))   
            },
            None => None,
        }
    }

    pub fn code(&self) -> String {
        self.code.to_string()
    }

    pub fn value(&self) -> String {
        self.value_quantity
            .as_ref()
            .and_then(|v| v.try_to_string())
            .unwrap_or_default()
    }

    pub fn interpretation(&self) -> String {
        self.interpretation
            .iter()
            .flatten()
            .map(|interpretation| interpretation.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation
    #[rustfmt::skip]
    pub fn interpretation_chip(&self) -> Option<Chip> {
        match self
            .interpretation
            .iter()
            .flatten()
            .find_map(|interpretation| {
                interpretation
                    .code_in_system("http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation")
            })?
            .as_str()
        {
            "N" => Some(Chip::new("bg-green-100 border-green-500", "Normal", "The result or observation value is within the reference range or expected norm.")),
            "A" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Abnormal", "The result or observation value is outside the reference range or expected norm.")),
            "H" => Some(Chip::new("bg-orange-100 border-orange-500", "High", "The result for a quantitative observation is above the upper limit of the reference range.")),
            "HU" => Some(Chip::new("bg-orange-100 border-orange-500", "Significantly high", "A test result that is significantly higher than the reference or therapeutic interval.")),
            "HH" => Some(Chip::new("bg-red-100 border-red-500", "Critical high", "The result is above a reference level at which immediate action should be considered for patient safety.")),
            "L" => Some(Chip::new("bg-blue-100 border-blue-500", "Low", "The result for a quantitative observation is below the lower limit of the reference range.")),
            "LU" => Some(Chip::new("bg-blue-100 border-blue-500", "Significantly low", "A test result that is significantly lower than the reference or therapeutic interval.")),
            "LL" => Some(Chip::new("bg-red-100 border-red-500", "Critical low", "The result is below a reference level at which immediate action should be considered for patient safety.")),
            "AA" => Some(Chip::new("bg-red-100 border-red-500", "Critical abnormal", "The result is outside a reference range at which immediate action should be considered for patient safety.")),
            "B" => Some(Chip::new("bg-green-100 border-green-500", "Better", "The current result has improved compared to the previous result.")),
            "W" => Some(Chip::new("bg-red-100 border-red-500", "Worse", "The current result has degraded compared to the previous result.")),
            "U" => Some(Chip::new("bg-orange-100 border-orange-500", "Significant change up", "The current result has increased from the previous result for a quantitative observation.")),
            "D" => Some(Chip::new("bg-blue-100 border-blue-500", "Significant change down", "The current result has decreased from the previous result for a quantitative observation.")),
            "POS" => Some(Chip::new("bg-red-100 border-red-500", "Positive", "A presence finding of the specified component based on the established threshold.")),
            "NEG" => Some(Chip::new("bg-green-100 border-green-500", "Negative", "An absence finding of the specified component based on the established threshold.")),
            "DET" => Some(Chip::new("bg-red-100 border-red-500", "Detected", "The measurement above the limit of detection of the performed test or procedure.")),
            "ND" => Some(Chip::new("bg-green-100 border-green-500", "Not detected", "The presence could not be determined within the limit of detection.")),
            "IND" => Some(Chip::new("bg-gray-100 border-gray-500", "Indeterminate", "The component could neither be declared positive/negative nor detected/not detected.")),
            "E" => Some(Chip::new("bg-gray-100 border-gray-500", "Equivocal", "The results are borderline and can neither be declared positive/negative nor detected/not detected.")),
            "S" => Some(Chip::new("bg-green-100 border-green-500", "Susceptible", "Bacterial strain inhibited by concentration associated with high likelihood of therapeutic success.")),
            "I" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Intermediate", "Bacterial strain inhibited by concentration associated with uncertain therapeutic effect.")),
            "R" => Some(Chip::new("bg-red-100 border-red-500", "Resistant", "Bacterial strain inhibited by concentration associated with high likelihood of therapeutic failure.")),
            "SDD" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Susceptible-dose dependent", "Isolates with MICs that approach usually attainable blood and tissue levels.")),
            "NS" => Some(Chip::new("bg-red-100 border-red-500", "Non-susceptible", "A category used for isolates for which only a susceptible interpretive criterion has been designated.")),
            "RR" => Some(Chip::new("bg-red-100 border-red-500", "Reactive", "The component reacted with the reagent above the reliably measurable limit.")),
            "WR" => Some(Chip::new("bg-yellow-100 border-yellow-500", "Weakly reactive", "The component reacted with the reagent, but below the reliably measurable limit.")),
            "NR" => Some(Chip::new("bg-green-100 border-green-500", "Non-reactive", "The component did not react measurably with the reagent.")),
            "CAR" => Some(Chip::new("bg-purple-100 border-purple-500", "Carrier", "The patient is considered as carrier based on the testing results.")),
            "<" => Some(Chip::new("bg-gray-100 border-gray-500", "Off scale low", "The result is below the minimum detection limit.")),
            ">" => Some(Chip::new("bg-gray-100 border-gray-500", "Off scale high", "The result is above the maximum quantifiable limit.")),
            "IE" => Some(Chip::new("bg-gray-100 border-gray-500", "Insufficient evidence", "There is insufficient evidence for a categorical interpretation.")),
            "EXP" => Some(Chip::new("bg-green-100 border-green-500", "Expected", "This result is determined to be Expected in light of known contraindicators.")),
            "UNE" => Some(Chip::new("bg-red-100 border-red-500", "Unexpected", "This result is determined to be Unexpected in light of known contraindicators.")),
            "EX" => Some(Chip::new("bg-gray-100 border-gray-500", "Outside threshold", "The observation/test result is interpreted as being outside the inclusion range for a particular protocol.")),
            "HX" => Some(Chip::new("bg-orange-100 border-orange-500", "Above high threshold", "The observation/test result is above the high threshold for a particular protocol.")),
            "LX" => Some(Chip::new("bg-blue-100 border-blue-500", "Below low threshold", "The observation/test result is below the low threshold for a particular protocol.")),
            "SYN-S" => Some(Chip::new("bg-green-100 border-green-500", "Synergy - susceptible", "The bacteria are susceptible to a combination therapy.")),
            "SYN-R" => Some(Chip::new("bg-red-100 border-red-500", "Synergy - resistant", "The bacteria are not susceptible to a combination therapy.")),
            "NCL" => Some(Chip::new("bg-gray-100 border-gray-500", "No CLSI defined breakpoint", "Not enough clinical trial data available to establish the breakpoints.")),
            _ => None,
        }
    }

    pub fn normal_range(&self) -> Option<String> {
        self.reference_range
            .iter()
            .flatten()
            .find(|r| {
                r.r#type.as_ref().is_some_and(|c| {
                    c.code_in_system("http://terminology.hl7.org/CodeSystem/referencerange-meaning")
                        == Some("normal".into())
                })
            })
            .map(|r| {
                format!(
                    "{} - {}",
                    r.low
                        .as_ref()
                        .and_then(|l| l.try_to_string())
                        .unwrap_or_default(),
                    r.high
                        .as_ref()
                        .and_then(|h| h.try_to_string())
                        .unwrap_or_default()
                )
            })
    }

    pub fn timeline_timestamp(&self) -> Option<jiff::Timestamp> {
        self.effective_date_time
    }
}

// pub trait TimelineEvent {
//     /// Returns the timestamp that is used to sort events in the timeline. If
//     /// `None` is returned, the event will not be included in the timeline.
//     fn timestamp(&self) -> Option<jiff::Timestamp>;

//     fn formatted_timestamp(&self) -> String {
//         self.timestamp()
//             .map(format_time)
//             .unwrap_or_else(|| "Unknown".to_string())
//     }
// }

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
    Observation(Observation),
    #[serde(other)]
    Unknown,
}

impl Resource {
    /// Returns the timestamp that is used to sort entries in the timeline.
    /// If `None` is returned, the entry will not be included in the timeline.
    pub fn timeline_timestamp(&self) -> Option<jiff::Timestamp> {
        match self {
            Resource::Condition(condition) => condition.timeline_timestamp(),
            Resource::Procedure(procedure) => procedure.timeline_timestamp(),
            Resource::Observation(observation) => observation.timeline_timestamp(),
            Resource::Patient(_) | Resource::Encounter(_) | Resource::Unknown => None,
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
