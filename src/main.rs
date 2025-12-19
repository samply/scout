use crate::controller::App;

mod controller;
mod fhir;
mod utils;

#[cfg(feature = "server")]
use dioxus::prelude::{DioxusRouterExt, dioxus_server};

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    dioxus::logger::initialize_default();
    std::sync::LazyLock::force(&utils::config::CONFIG);
    let addr = dioxus::cli_config::fullstack_address_or_localhost();

    let router = axum::Router::new()
        .serve_dioxus_application(dioxus_server::ServeConfig::new(), crate::controller::App);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}
