use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::components::error_display::ErrorDisplay;
use crate::pages::{
    AddReceiptPage, AddTripPage, EditReceiptPage, ReceiptDetailPage, SettingsPage, TripDetailPage,
    TripListPage,
};
use crate::state::AppStore;
use crate::storage::categories::get_all_categories;
use crate::storage::db::{open_database, seed_categories};
use crate::storage::trips::get_all_trips;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    TripList,
    #[at("/trip/new")]
    AddTrip,
    #[at("/trip/:id")]
    TripDetail { id: String },
    #[at("/trip/:id/receipt/new")]
    AddReceipt { id: String },
    #[at("/trip/:id/receipt/:rid/edit")]
    EditReceipt { id: String, rid: String },
    #[at("/trip/:id/receipt/:rid")]
    ReceiptDetail { id: String, rid: String },
    #[at("/settings")]
    Settings,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: Route) -> Html {
    match route {
        Route::TripList => html! { <TripListPage /> },
        Route::AddTrip => html! { <AddTripPage /> },
        Route::TripDetail { id } => html! { <TripDetailPage trip_id={id} /> },
        Route::AddReceipt { id } => html! { <AddReceiptPage trip_id={id} /> },
        Route::EditReceipt { id, rid } => {
            html! { <EditReceiptPage trip_id={id} receipt_id={rid} /> }
        }
        Route::ReceiptDetail { id, rid } => {
            html! { <ReceiptDetailPage trip_id={id} receipt_id={rid} /> }
        }
        Route::Settings => html! { <SettingsPage /> },
        Route::NotFound => html! { <div class="page-center"><h2>{"Page not found"}</h2></div> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let (store, dispatch) = use_store::<AppStore>();

    // Initialize DB on first mount
    use_effect_with((), move |_| {
        let dispatch = dispatch.clone();
        spawn_local(async move {
            dispatch.reduce_mut(|s| s.loading = true);

            match open_database().await {
                Ok(db) => {
                    let db = Rc::new(db);
                    // Seed categories if needed
                    if let Err(e) = seed_categories(&db).await {
                        log::warn!("Seed categories failed: {}", e);
                    }
                    // Load categories
                    let categories = get_all_categories(&db).await.unwrap_or_default();
                    // Load trips
                    let trips = get_all_trips(&db).await.unwrap_or_default();

                    dispatch.reduce_mut(|s| {
                        s.db = Some(db);
                        s.categories = categories;
                        s.trips = trips;
                        s.loading = false;
                    });
                }
                Err(e) => {
                    dispatch.reduce_mut(|s| {
                        s.error = Some(format!("Failed to open database: {}", e));
                        s.loading = false;
                    });
                }
            }
        });
        || ()
    });

    // Register service worker
    use_effect_with((), |_| {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            if let Ok(sw) = js_sys::Reflect::get(&navigator, &"serviceWorker".into())
                && !sw.is_undefined()
                && !sw.is_null()
            {
                let sw_container: web_sys::ServiceWorkerContainer =
                    wasm_bindgen::JsCast::unchecked_into(sw);
                let _ = sw_container.register("/sw.js");
            }
        }
        || ()
    });

    html! {
        <BrowserRouter basename="/receipt-tracker/">
            <div class="app">
                if store.loading {
                    <div class="loading-screen">
                        <div class="spinner" />
                        <p>{"Loading..."}</p>
                    </div>
                } else if let Some(err) = &store.error {
                    <ErrorDisplay message={err.clone()} />
                } else {
                    <Switch<Route> render={switch} />
                }
            </div>
        </BrowserRouter>
    }
}
