use wasm_bindgen::prelude::*;

mod app;
mod components;
mod error;
mod export;
mod models;
mod pages;
mod state;
mod storage;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logger");
    log::info!("Receipt Tracker starting...");
    yew::Renderer::<app::App>::new().render();
}
