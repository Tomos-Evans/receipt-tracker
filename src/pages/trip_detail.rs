use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::app::Route;
use crate::components::app_bar::AppBar;
use crate::components::fab::Fab;
use crate::components::receipt_card::ReceiptCard;
use crate::state::AppStore;
use crate::storage::receipts::{get_receipts_for_trip, delete_receipts_for_trip};
use crate::storage::trips::delete_trip;

#[derive(Properties, PartialEq)]
pub struct TripDetailPageProps {
    pub trip_id: String,
}

#[function_component(TripDetailPage)]
pub fn trip_detail_page(props: &TripDetailPageProps) -> Html {
    let (store, dispatch) = use_store::<AppStore>();
    let navigator = use_navigator().unwrap();
    let trip_id = props.trip_id.clone();

    // Find the current trip
    let trip = store.trips.iter().find(|t| t.id == trip_id).cloned();

    // Load receipts for this trip on mount / trip_id change
    {
        let trip_id = trip_id.clone();
        let dispatch = dispatch.clone();
        let db = store.db.clone();
        use_effect_with(trip_id.clone(), move |_| {
            if let Some(db) = db {
                spawn_local(async move {
                    match get_receipts_for_trip(&db, &trip_id).await {
                        Ok(receipts) => {
                            dispatch.reduce_mut(|s| s.current_receipts = receipts);
                        }
                        Err(e) => {
                            dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                        }
                    }
                });
            }
            || ()
        });
    }

    let on_add_receipt = {
        let nav = navigator.clone();
        let tid = trip_id.clone();
        Callback::from(move |_| nav.push(&Route::AddReceipt { id: tid.clone() }))
    };

    let on_export_csv = {
        let store = store.clone();
        let trip_id = trip_id.clone();
        Callback::from(move |_: MouseEvent| {
            let receipts = store.current_receipts.clone();
            let trip = store.trips.iter().find(|t| t.id == trip_id).cloned();
            let categories = store.categories.clone();
            if let Some(trip) = trip {
                crate::export::csv::export_csv(&trip, &receipts, &categories);
            }
        })
    };

    let on_export_pdf = {
        let store = store.clone();
        let trip_id = trip_id.clone();
        let db = store.db.clone();
        let dispatch = dispatch.clone();
        Callback::from(move |_: MouseEvent| {
            let receipts = store.current_receipts.clone();
            let trip = store.trips.iter().find(|t| t.id == trip_id).cloned();
            let categories = store.categories.clone();
            if let Some(trip) = trip
                && let Some(db) = &db
            {
                let db = Rc::clone(db);
                let dispatch = dispatch.clone();
                spawn_local(async move {
                    if let Err(e) = crate::export::pdf::export_pdf(&db, &trip, &receipts, &categories).await {
                        dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                    }
                });
            }
        })
    };

    let on_delete_trip = {
        let store = store.clone();
        let dispatch = dispatch.clone();
        let navigator = navigator.clone();
        let trip_id = trip_id.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(db) = &store.db {
                let db = Rc::clone(db);
                let trip_id = trip_id.clone();
                let dispatch = dispatch.clone();
                let navigator = navigator.clone();
                spawn_local(async move {
                    let _ = delete_receipts_for_trip(&db, &trip_id).await;
                    match delete_trip(&db, &trip_id).await {
                        Ok(()) => {
                            dispatch.reduce_mut(|s| {
                                s.trips.retain(|t| t.id != trip_id);
                                s.current_receipts.clear();
                            });
                            navigator.push(&Route::TripList);
                        }
                        Err(e) => {
                            dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                        }
                    }
                });
            }
        })
    };

    let total: f64 = store.current_receipts.iter().map(|r| r.amount).sum();
    let currency = trip.as_ref().map(|t| t.currency.as_str()).unwrap_or("USD");

    let actions = html! {
        <>
            <button class="icon-btn" onclick={on_export_csv} title="Export CSV">
                <span class="material-icons">{"table_view"}</span>
            </button>
            <button class="icon-btn" onclick={on_export_pdf} title="Export PDF">
                <span class="material-icons">{"picture_as_pdf"}</span>
            </button>
            <button class="icon-btn danger" onclick={on_delete_trip} title="Delete Trip">
                <span class="material-icons">{"delete"}</span>
            </button>
        </>
    };

    html! {
        <div class="page">
            <AppBar
                title={trip.as_ref().map(|t| t.name.clone()).unwrap_or_default()}
                show_back=true
                actions={actions}
            />
            <main class="page-content">
                if let Some(trip) = &trip {
                    <div class="trip-summary-bar">
                        <span>{ format!("{} – {}", trip.start_date, trip.end_date) }</span>
                        <strong>{ format!("Total: {} {:.2}", currency, total) }</strong>
                    </div>
                }
                if store.current_receipts.is_empty() {
                    <div class="empty-state">
                        <span class="material-icons empty-icon">{"receipt_long"}</span>
                        <h2>{"No receipts yet"}</h2>
                        <p>{"Tap + to add your first receipt"}</p>
                    </div>
                } else {
                    <div class="card-list">
                        { for store.current_receipts.iter().map(|receipt| {
                            let cat = store.categories.iter()
                                .find(|c| c.id == receipt.category_id)
                                .cloned();
                            html! {
                                <ReceiptCard
                                    key={receipt.id.clone()}
                                    receipt={receipt.clone()}
                                    category={cat}
                                    currency={currency.to_string()}
                                    has_photo={false}
                                />
                            }
                        })}
                    </div>
                }
            </main>
            <Fab icon="add" label="Add Receipt" onclick={on_add_receipt} />
        </div>
    }
}
