use wasm_bindgen::prelude::*;

mod app;
mod error;
mod models;
mod storage;
mod state;
mod pages;
mod components;
mod export;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logger");
    log::info!("Receipt Tracker starting...");
    yew::Renderer::<app::App>::new().render();
}
