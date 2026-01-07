use crate::fhir::code::{Annotation, Chip, CodeableConcept, Identifier, Reference};
use crate::fhir::procedure::{Quantity, SimpleQuantity};
use serde::{Deserialize, Serialize};
use std::fmt;

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
        self.identifier.as_ref()?.iter().find_map(|id| {
            let is_obi = id
                .r#type
                .as_ref()
                .and_then(|t| t.code_in_system("http://terminology.hl7.org/CodeSystem/v2-0203"))
                .as_deref()
                == Some("OBI");

            if is_obi { id.value.clone() } else { None }
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

    pub fn timeline_timestamp(&self) -> Option<jiff::Timestamp> { self.effective_date_time }
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
