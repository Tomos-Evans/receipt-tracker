use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::app::Route;
use crate::components::app_bar::AppBar;
use crate::components::trip_form::{TripForm, TripFormData};
use crate::models::Trip;
use crate::state::AppStore;
use crate::storage::trips::save_trip;

#[function_component(AddTripPage)]
pub fn add_trip_page() -> Html {
    let (store, dispatch) = use_store::<AppStore>();
    let navigator = use_navigator().unwrap();
    let form_data = use_state(TripFormData::default);

    let on_change = {
        let form_data = form_data.clone();
        Callback::from(move |data| form_data.set(data))
    };

    let on_submit = {
        let store = store.clone();
        let dispatch = dispatch.clone();
        let navigator = navigator.clone();
        Callback::from(move |data: TripFormData| {
            let db = match store.db.as_ref() {
                Some(db) => Rc::clone(db),
                None => return,
            };
            let dispatch = dispatch.clone();
            let navigator = navigator.clone();
            let trip = Trip::new(
                data.name.clone(),
                data.currency.clone(),
                data.start_naive().unwrap(),
                data.end_naive().unwrap(),
            );

            spawn_local(async move {
                match save_trip(&db, &trip).await {
                    Ok(()) => {
                        let trip_id = trip.id.clone();
                        dispatch.reduce_mut(|s| s.trips.insert(0, trip));
                        navigator.push(&Route::TripDetail { id: trip_id });
                    }
                    Err(e) => {
                        dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                    }
                }
            });
        })
    };

    html! {
        <div class="page">
            <AppBar title="New Trip" show_back=true />
            <main class="page-content">
                <TripForm
                    data={(*form_data).clone()}
                    on_change={on_change}
                    on_submit={on_submit}
                    submit_label="Create Trip"
                />
            </main>
        </div>
    }
}
