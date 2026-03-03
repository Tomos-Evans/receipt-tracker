use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::app::Route;
use crate::components::app_bar::AppBar;
use crate::components::fab::Fab;
use crate::components::trip_card::TripCard;
use crate::state::AppStore;
use crate::storage::receipts::get_all_receipts;

/// Maps trip_id → (receipt_count, total_amount)
type TripSummaries = HashMap<String, (usize, f64)>;

#[function_component(TripListPage)]
pub fn trip_list_page() -> Html {
    let (store, _) = use_store::<AppStore>();
    let navigator = use_navigator().unwrap();
    let summaries: UseStateHandle<TripSummaries> = use_state(HashMap::new);

    // Load receipt summaries whenever the DB is ready or trips change
    {
        let db = store.db.clone();
        let summaries = summaries.clone();
        use_effect_with(store.trips.clone(), move |_| {
            if let Some(db) = db {
                spawn_local(async move {
                    if let Ok(all_receipts) = get_all_receipts(&db).await {
                        let mut map: TripSummaries = HashMap::new();
                        for r in &all_receipts {
                            let entry = map.entry(r.trip_id.clone()).or_insert((0, 0.0));
                            entry.0 += 1;
                            entry.1 += r.amount;
                        }
                        summaries.set(map);
                    }
                });
            }
            || ()
        });
    }

    let on_add = {
        let nav = navigator.clone();
        Callback::from(move |_| nav.push(&Route::AddTrip))
    };

    html! {
        <div class="page">
            <AppBar title="My Trips" />
            <main class="page-content">
                if store.trips.is_empty() {
                    <div class="empty-state">
                        <span class="material-icons empty-icon">{"luggage"}</span>
                        <h2>{"No trips yet"}</h2>
                        <p>{"Tap + to create your first trip"}</p>
                    </div>
                } else {
                    <div class="card-list">
                        { for store.trips.iter().map(|trip| {
                            let (count, total) = summaries
                                .get(&trip.id)
                                .copied()
                                .unwrap_or((0, 0.0));
                            html! {
                                <TripCard
                                    key={trip.id.clone()}
                                    trip={trip.clone()}
                                    receipt_count={count}
                                    total={total}
                                />
                            }
                        })}
                    </div>
                }
            </main>
            <Fab icon="add" label="New Trip" onclick={on_add} />
        </div>
    }
}
