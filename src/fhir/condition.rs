use crate::fhir::code::{Annotation, Chip, CodeableConcept, Period, Reference};
use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub recorded_date: jiff::Timestamp,
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
        Some(self.recorded_date)
    }

    pub fn is_neoplasm(&self) -> bool {
        self.code
            .code_in_system("http://fhir.de/CodeSystem/bfarm/icd-10-gm")
            .is_some_and(|c| "C00".to_string() <= c && c <= "D48.9".to_string())
    }
}
