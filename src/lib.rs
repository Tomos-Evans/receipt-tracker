use wasm_bindgen::prelude::*;

mod app;
mod components;
mod error;
mod export;
mod models;
mod pages;
mod state;
mod storage;

// The test runner supplies its own entry point, so suppress ours to avoid a
// "entry symbol `main` declared multiple times" link error.
#[cfg(not(test))]
#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logger");
    log::info!("Receipt Tracker starting...");
    yew::Renderer::<app::App>::new().render();
}
