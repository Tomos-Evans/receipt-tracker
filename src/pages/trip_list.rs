use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::app::Route;
use crate::components::app_bar::AppBar;
use crate::components::fab::Fab;
use crate::components::trip_card::TripCard;
use crate::state::AppStore;

#[function_component(TripListPage)]
pub fn trip_list_page() -> Html {
    let (store, _) = use_store::<AppStore>();
    let navigator = use_navigator().unwrap();

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
                            // Simple totals — in real usage you'd load these from the store
                            html! {
                                <TripCard
                                    key={trip.id.clone()}
                                    trip={trip.clone()}
                                    receipt_count={0}
                                    total={0.0}
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
