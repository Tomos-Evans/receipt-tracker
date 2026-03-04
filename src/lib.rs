// Yew components are constructed exclusively via the html! macro, which
// rust-analyzer doesn't trace for dead-code analysis. Suppress the resulting
// false-positive warnings across the whole crate.
#![allow(unused)]
use wasm_bindgen::prelude::*;

mod app;
mod components;
mod error;
mod export;
mod models;
mod pages;
mod state;
mod storage;

// In test builds the wasm-pack runner supplies its own start symbol, so we
// drop the #[wasm_bindgen(start)] attribute to avoid a duplicate-symbol link
// error — while keeping the function itself so rust-analyzer doesn't flag all
// the app code as unreachable.
#[cfg_attr(not(test), wasm_bindgen(start))]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logger");
    log::info!("Receipt Tracker starting...");
    yew::Renderer::<app::App>::new().render();
}
