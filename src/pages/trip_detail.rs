use gloo_timers::callback::Timeout;
use std::collections::BTreeMap;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

type CatRow = (String, Option<String>, Option<String>, f64);

use crate::app::Route;
use crate::components::app_bar::AppBar;
use crate::components::fab::Fab;
use crate::components::receipt_card::ReceiptCard;
use crate::state::AppStore;
use crate::storage::receipts::{delete_receipts_for_trip, get_receipts_for_trip};
use crate::storage::trips::delete_trip;

#[derive(Properties, PartialEq)]
pub struct TripDetailPageProps {
    pub trip_id: String,
}

#[function_component(TripDetailPage)]
pub fn trip_detail_page(props: &TripDetailPageProps) -> Html {
    let (store, dispatch) = use_store::<AppStore>();
    let navigator = use_navigator().expect("TripDetailPage must be rendered inside a Router");
    let trip_id = props.trip_id.clone();
    let show_toast = use_state(|| false);

    let on_back = {
        let nav = navigator.clone();
        Callback::from(move |_| nav.push(&Route::TripList))
    };

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

    let on_export_pdf = {
        let store = store.clone();
        let trip_id = trip_id.clone();
        let db = store.db.clone();
        let dispatch = dispatch.clone();
        let show_toast = show_toast.clone();
        Callback::from(move |_: MouseEvent| {
            let receipts = store.current_receipts.clone();
            let trip = store.trips.iter().find(|t| t.id == trip_id).cloned();
            let categories = store.categories.clone();
            if let Some(trip) = trip
                && let Some(db) = &db
            {
                let db = Rc::clone(db);
                let dispatch = dispatch.clone();
                let show_toast = show_toast.clone();
                spawn_local(async move {
                    match crate::export::pdf::export_pdf(&db, &trip, &receipts, &categories).await {
                        Ok(()) => {
                            show_toast.set(true);
                            let show_toast = show_toast.clone();
                            Timeout::new(3000, move || show_toast.set(false)).forget();
                        }
                        Err(e) => {
                            dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                        }
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

    // Currency totals: currency → total
    let mut currency_totals: BTreeMap<String, f64> = BTreeMap::new();
    for r in &store.current_receipts {
        *currency_totals.entry(r.currency.clone()).or_default() += r.amount;
    }
    let is_multi_currency = currency_totals.len() > 1;

    let total_str: String = currency_totals
        .iter()
        .map(|(c, a)| format!("{} {:.2}", c, a))
        .collect::<Vec<_>>()
        .join("  ·  ");

    // Category breakdown grouped by currency
    let mut cat_by_currency: BTreeMap<String, Vec<CatRow>> = BTreeMap::new();
    for cat in &store.categories {
        for cur in currency_totals.keys() {
            let cat_total: f64 = store
                .current_receipts
                .iter()
                .filter(|r| r.category_id == cat.id && &r.currency == cur)
                .map(|r| r.amount)
                .sum();
            if cat_total > 0.0 {
                cat_by_currency.entry(cur.clone()).or_default().push((
                    cat.name.clone(),
                    cat.icon.clone(),
                    cat.color.clone(),
                    cat_total,
                ));
            }
        }
    }
    for rows in cat_by_currency.values_mut() {
        rows.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
    }

    let actions = html! {
        <>
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
                on_back={on_back}
                actions={actions}
            />
            <main class="page-content">
                if let Some(trip) = &trip {
                    <div class="trip-summary-bar">
                        <span>{ format!("{} – {}", trip.start_date, trip.end_date) }</span>
                        <strong>{ total_str }</strong>
                    </div>
                }
                if !cat_by_currency.is_empty() {
                    <div class="category-breakdown">
                        <div class="category-breakdown-header">{"By category"}</div>
                        { for cat_by_currency.iter().map(|(cur, rows)| {
                            let cur = cur.clone();
                            html! {
                                <>
                                    if is_multi_currency {
                                        <div class="currency-group-label">{ &cur }</div>
                                    }
                                    { for rows.iter().map(|(name, icon, color, amount)| {
                                        let icon_name = icon.as_deref().unwrap_or("label");
                                        let color_style = format!("color:{}", color.as_deref().unwrap_or("#757575"));
                                        html! {
                                            <div class="cat-breakdown-row">
                                                <span class="material-icons cat-breakdown-icon" style={color_style}>{icon_name}</span>
                                                <span class="cat-breakdown-name">{name}</span>
                                                <span class="cat-breakdown-amount">
                                                    { format!("{} {:.2}", cur, amount) }
                                                </span>
                                            </div>
                                        }
                                    })}
                                </>
                            }
                        })}
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
                                    has_photo={false}
                                />
                            }
                        })}
                    </div>
                }
            </main>
            <Fab icon="add" label="Add Receipt" onclick={on_add_receipt} />
            if *show_toast {
                <div class="toast">
                    <span class="material-icons">{"check_circle"}</span>
                    {"PDF downloaded"}
                </div>
            }
        </div>
    }
}
