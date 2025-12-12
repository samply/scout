use crate::fhir::code::{Annotation, Chip, CodeableConcept, Period};
use serde::{Deserialize, Serialize};

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
