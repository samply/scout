mod utils;
mod controller;
mod fhir;

#[cfg(feature = "server")]
fn main() {
    dioxus::logger::initialize_default();
    std::sync::LazyLock::force(&utils::config::CONFIG);

    dioxus::launch(controller::App);
}


#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(controller::App);
}