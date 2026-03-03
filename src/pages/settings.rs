use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::prelude::*;

use yew_router::prelude::*;
use crate::app::Route;
use crate::components::app_bar::AppBar;
use crate::models::Category;
use crate::state::AppStore;
use crate::storage::categories::{save_category, delete_category};

#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let (store, dispatch) = use_store::<AppStore>();
    let navigator = use_navigator().unwrap();
    let new_cat_name = use_state(String::new);

    let on_back = Callback::from(move |_| navigator.push(&Route::TripList));
    let new_cat_color = use_state(|| "#6750A4".to_string());

    let on_name_input = {
        let s = new_cat_name.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            s.set(input.value());
        })
    };

    let on_color_input = {
        let s = new_cat_color.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            s.set(input.value());
        })
    };

    let on_add_category = {
        let store = store.clone();
        let dispatch = dispatch.clone();
        let new_cat_name = new_cat_name.clone();
        let new_cat_color = new_cat_color.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let name = (*new_cat_name).trim().to_string();
            if name.is_empty() { return; }
            if let Some(db) = &store.db {
                let db = Rc::clone(db);
                let dispatch = dispatch.clone();
                let color = (*new_cat_color).clone();
                let new_cat_name = new_cat_name.clone();
                let cat = Category::new(name, None, Some(color));
                spawn_local(async move {
                    match save_category(&db, &cat).await {
                        Ok(()) => {
                            dispatch.reduce_mut(|s| s.categories.push(cat));
                            new_cat_name.set(String::new());
                        }
                        Err(e) => {
                            dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                        }
                    }
                });
            }
        })
    };

    let make_delete_cat = {
        let store = store.clone();
        let dispatch = dispatch.clone();
        move |cat_id: String| {
            let store = store.clone();
            let dispatch = dispatch.clone();
            Callback::from(move |_: MouseEvent| {
                if let Some(db) = &store.db {
                    let db = Rc::clone(db);
                    let dispatch = dispatch.clone();
                    let cat_id = cat_id.clone();
                    spawn_local(async move {
                        match delete_category(&db, &cat_id).await {
                            Ok(()) => {
                                dispatch.reduce_mut(|s| s.categories.retain(|c| c.id != cat_id));
                            }
                            Err(e) => {
                                dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                            }
                        }
                    });
                }
            })
        }
    };

    html! {
        <div class="page">
            <AppBar title="Settings" on_back={on_back} />
            <main class="page-content">
                <section class="settings-section">
                    <h2 class="section-title">{"Categories"}</h2>
                    <div class="category-list">
                        { for store.categories.iter().map(|cat| {
                            let delete_cb = make_delete_cat(cat.id.clone());
                            let color = cat.color.as_deref().unwrap_or("#757575");
                            html! {
                                <div class="category-row" key={cat.id.clone()}>
                                    <span class="cat-color-dot" style={format!("background:{}", color)} />
                                    <span class="material-icons" style={format!("color:{}", color)}>
                                        { cat.icon.as_deref().unwrap_or("circle") }
                                    </span>
                                    <span class="cat-name">{ &cat.name }</span>
                                    <button class="icon-btn danger" onclick={delete_cb}>
                                        <span class="material-icons">{"delete"}</span>
                                    </button>
                                </div>
                            }
                        })}
                    </div>

                    <form class="add-category-form" onsubmit={on_add_category}>
                        <h3>{"Add Custom Category"}</h3>
                        <div class="form-field">
                            <label class="form-label">{"Name"}</label>
                            <input
                                class="form-input"
                                type="text"
                                placeholder="Category name"
                                value={(*new_cat_name).clone()}
                                oninput={on_name_input}
                                required=true
                            />
                        </div>
                        <div class="form-field">
                            <label class="form-label">{"Color"}</label>
                            <input
                                class="form-input"
                                type="color"
                                value={(*new_cat_color).clone()}
                                oninput={on_color_input}
                            />
                        </div>
                        <button type="submit" class="btn btn-primary">{"Add Category"}</button>
                    </form>
                </section>

                <section class="settings-section">
                    <h2 class="section-title">{"About"}</h2>
                    <p class="about-text">
                        {"Receipt Tracker v0.1.0 — all data stored locally on your device."}
                    </p>
                </section>
            </main>
        </div>
    }
}
