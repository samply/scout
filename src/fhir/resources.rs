use crate::fhir::condition::Condition;
use crate::fhir::encounter::Encounter;
use crate::fhir::observation::Observation;
use crate::fhir::patient::Patient;
use crate::fhir::procedure::Procedure;
use serde::{Deserialize, Serialize};
use std::fmt;

// This module contains the data structures for the FHIR resources used in the application.
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
