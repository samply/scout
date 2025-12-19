use serde::{Deserialize, Serialize};
use std::fmt;
use crate::utils;

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
            display = utils::config::CODE_MAPS
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
