use dioxus::prelude::*;
use itertools::Itertools;

mod fhir;
mod server;
mod serverfn;
mod table;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    PatientTable {},
    #[route("/patient/:id")]
    PatientView { id: String },
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::logger::initialize_default();

    #[cfg(feature = "server")]
    if let Err(e) = server::load_config() {
        tracing::error!("Failed to load config: {e}");
        std::process::exit(1);
    }

    #[cfg(feature = "server")]
    if let Err(e) = server::load_code_maps() {
        tracing::error!("Failed to load code maps: {e}");
        std::process::exit(1);
    }

    dioxus::launch(App);
}

pub fn format_timestamp(timestamp: jiff::Timestamp) -> String {
    let zoned = timestamp.to_zoned(jiff::tz::TimeZone::system());
    // Jan 08, 2020, 07:00 CET
    zoned.strftime("%b %d, %Y, %H:%M %Z").to_string()
}

#[component]
fn App() -> Element {
    // Load polyfill for CSS anchor positioning if needed (https://github.com/oddbird/css-anchor-positioning)
    document::eval(
        "if (!('anchorName' in document.documentElement.style)) import('https://unpkg.com/@oddbird/css-anchor-positioning/dist/css-anchor-positioning-fn.js').then(mod => {window.CSSAnchorPositioning = mod.default; window.CSSAnchorPositioning()});",
    );

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn PatientTable() -> Element {
    let patients = use_server_future(|| serverfn::get_patients())?;
    match &*patients.read_unchecked() {
        Some(Ok(patients)) => rsx! {
            table::Table {
                columns: vec![
                    table::Column::new("ID").hidden(),
                    table::Column::new("Gender").categorical(),
                    table::Column::new("Birth Date"),
                    table::Column::new("Deceased").categorical(),
                    table::Column::new("Address"),
                ],
                data: patients
                    .iter()
                    .map(|p| {
                        vec![p.id.to_string(), p.gender(), p.birth_date(), p.deceased(), p.address()]
                    })
                    .collect(),
                ondetail: {
                    let patients = patients.clone();
                    move |idx: usize| {
                        let id = patients[idx].id.to_string();
                        navigator().push(Route::PatientView { id });
                    }
                },
            }
        },
        Some(Err(e)) => rsx! { "Error loading patients: {e:#}" },
        None => rsx! { "Loading..." },
    }
}

#[component]
fn OptionalChip(chip: Option<fhir::Chip>) -> Element {
    rsx! {
        if let Some(chip) = chip {
            " "
            span {
                class: "inline-block text-sm border rounded-full px-1.5 {chip.class}",
                title: "{chip.hover_text}",
                "{chip.text}"
            }
        }
    }
}

#[component]
fn CodeableConcept(codeable_concept: fhir::CodeableConcept) -> Element {
    // Put user-selected coding first
    let codings = codeable_concept
        .coding
        .iter()
        .flatten()
        .sorted_by_key(|c| c.user_selected)
        .rev()
        .collect::<Vec<_>>();

    let text = codeable_concept
        .text
        .or_else(|| {
            codings
                .first()
                .and_then(|c| c.display.clone().or(c.code.clone()))
        })
        .unwrap_or_default();

    rsx! {
        "{text}"
        for coding in codings {
            " "
            span {
                class: "inline-block text-sm border rounded-full px-1.5 bg-blue-100 border-blue-500",
                title: "{coding.system.clone().unwrap_or_default()}",
                "{coding.code.clone().unwrap_or_default()}"
            }
        }
    }
}

#[component]
fn PatientView(id: String) -> Element {
    let id = use_signal(|| id);
    let patient_details = use_server_future(move || serverfn::get_patient_details(id()))?;
    match &*patient_details.read_unchecked() {
        Some(Ok((patient, bundle))) => rsx! {
            div { class: "max-w-4xl mx-auto px-4",
                h2 { class: "text-xl font-bold my-3", "Patient Details" }
                p { "Gender: {patient.gender()}" }
                p { "Birth Date: {patient.birth_date()}" }
                p { "Deceased: {patient.deceased()}" }
                p { "Address: {patient.address()}" }
                h2 { class: "text-xl font-bold my-3", "Patient Timeline" }
                for ((entry , timestamp) , (_ , next_timestamp)) in bundle
                    .entry
                    .iter()
                    .filter_map(|e| e.resource.timeline_timestamp().map(|t| (e, t)))
                    .sorted_by_key(|(_, t)| t.clone())
                    .circular_tuple_windows()
                {
                    match entry.resource {
                        fhir::Resource::Encounter(ref encounter) => {
                            rsx! {
                                div { class: "my-3 p-2 border rounded border-gray-300 bg-gray-50",
                                    p {
                                        span { class: "font-bold", "Encounter" }
                                        OptionalChip { chip: encounter.status_chip() }
                                    }
                                    time { class: "text-sm text-gray-600", "{format_timestamp(timestamp)}" }
                                    p { "Class: {encounter.class()}" }
                                    p { "Visit number: {encounter.visit_number()}" }
                                    p { "Encounter level: {encounter.encounter_level()}" }
                                    p { "Service type: {encounter.service_type()}" }
                                    p { "Service provider: {encounter.service_provider()}" }
                                }
                            }
                        }
                        fhir::Resource::Condition(ref condition) => {
                            rsx! {
                                div {
                                    class: "my-3 p-2 border rounded",
                                    class: if condition.is_neoplasm() { "border-orange-300 bg-orange-50" } else { "border-gray-300 bg-gray-50" },
                                    div { class: "flex items-center flex-wrap gap-1",
                                        span { class: "font-bold", "⚕️ Condition" }
                                        OptionalChip { chip: condition.clinical_status_chip() }
                                        OptionalChip { chip: condition.verification_status_chip() }
                                        if condition.is_neoplasm() {
                                            div {
                                                class: "ml-auto",
                                                title: "ICD-10-GM codes in the range C00-D48 represent neoplasms and are highlighted orange.",
                                                svg {
                                                    class: "size-5 text-orange-300",
                                                    xmlns: "http://www.w3.org/2000/svg",
                                                    "viewBox": "0 0 24 24",
                                                    fill: "currentColor",
                                                    path {
                                                        "fill-rule": "evenodd",
                                                        d: "M2 12C2 6.477 6.477 2 12 2s10 4.477 10 10-4.477 10-10 10S2 17.523 2 12Zm9.008-3.018a1.502 1.502 0 0 1 2.522 1.159v.024a1.44 1.44 0 0 1-1.493 1.418 1 1 0 0 0-1.037.999V14a1 1 0 1 0 2 0v-.539a3.44 3.44 0 0 0 2.529-3.256 3.502 3.502 0 0 0-7-.255 1 1 0 0 0 2 .076c.014-.398.187-.774.48-1.044Zm.982 7.026a1 1 0 1 0 0 2H12a1 1 0 1 0 0-2h-.01Z",
                                                        "clip-rule": "evenodd",
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    time { class: "text-sm text-gray-600", "{format_timestamp(timestamp)}" }
                                    p {
                                        "Code: "
                                        CodeableConcept { codeable_concept: condition.code.clone() }
                                    }
                                }
                            }
                        }
                        fhir::Resource::Procedure(ref procedure) => {
                            rsx! {
                                div {
                                    class: "my-3 p-2 border rounded",
                                    class: if procedure.is_radiation_therapy_or_nuclear_medicine_therapy_or_chemotherapy() { "border-orange-300 bg-orange-50" } else { "border-gray-300 bg-gray-50" },
                                    div { class: "flex items-center flex-wrap gap-1",
                                        span { class: "font-bold", "💉 Procedure" }
                                        OptionalChip { chip: procedure.status_chip() }
                                        if procedure.is_radiation_therapy_or_nuclear_medicine_therapy_or_chemotherapy() {
                                            div {
                                                class: "ml-auto",
                                                title: "OPS codes in the range 8-52...8-54 represent radiation therapy, nuclear medicine therapy, and chemotherapy and are highlighted orange.",
                                                svg {
                                                    class: "size-5 text-orange-300",
                                                    xmlns: "http://www.w3.org/2000/svg",
                                                    "viewBox": "0 0 24 24",
                                                    fill: "currentColor",
                                                    path {
                                                        "fill-rule": "evenodd",
                                                        d: "M2 12C2 6.477 6.477 2 12 2s10 4.477 10 10-4.477 10-10 10S2 17.523 2 12Zm9.008-3.018a1.502 1.502 0 0 1 2.522 1.159v.024a1.44 1.44 0 0 1-1.493 1.418 1 1 0 0 0-1.037.999V14a1 1 0 1 0 2 0v-.539a3.44 3.44 0 0 0 2.529-3.256 3.502 3.502 0 0 0-7-.255 1 1 0 0 0 2 .076c.014-.398.187-.774.48-1.044Zm.982 7.026a1 1 0 1 0 0 2H12a1 1 0 1 0 0-2h-.01Z",
                                                        "clip-rule": "evenodd",
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    time { class: "text-sm text-gray-600", "{format_timestamp(timestamp)}" }
                                    p {
                                        "Code: "
                                        CodeableConcept { codeable_concept: procedure.code.clone() }
                                    }
                                    p { "Category: {procedure.category()}" }
                                }
                            }
                        }
                        fhir::Resource::Observation(ref observation) => {
                            rsx! {
                                div { class: "my-3 p-2 border rounded border-gray-300 bg-gray-50",
                                    p {
                                        span { class: "font-bold", "🧪 Lab result" }
                                        OptionalChip { chip: observation.status_chip() }
                                    }
                                    time { class: "text-sm text-gray-600", "{format_timestamp(timestamp)}" }
                                    p {
                                        "Code: "
                                        CodeableConcept { codeable_concept: observation.code.clone() }
                                    }
                                    p {
                                        "Value: {observation.value()} "
                                        OptionalChip { chip: observation.interpretation_chip() }
                                    }
                                    if let Some(normal_range) = observation.normal_range() {
                                        p { "Normal range: {normal_range}" }
                                    }
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                    if next_timestamp.to_zoned(jiff::tz::TimeZone::system()).date()
                        > timestamp.to_zoned(jiff::tz::TimeZone::system()).date()
                    {
                        h3 { class: "my-2 text-center",
                            "{next_timestamp.to_zoned(jiff::tz::TimeZone::system()).date().strftime(\"%b %d, %Y\")}"
                        }
                    }
                }
            }
        },
        Some(Err(e)) => rsx! { "Error loading patient: {e:#}" },
        None => rsx! { "Loading..." },
    }
}
