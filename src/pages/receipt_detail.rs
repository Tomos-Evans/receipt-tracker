use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::app::Route;
use crate::components::app_bar::AppBar;
use crate::components::photo_viewer::PhotoViewer;
use crate::state::AppStore;
use crate::storage::photos::{delete_photo, get_photo};
use crate::storage::receipts::delete_receipt;

#[derive(Properties, PartialEq)]
pub struct ReceiptDetailPageProps {
    pub trip_id: String,
    pub receipt_id: String,
}

#[function_component(ReceiptDetailPage)]
pub fn receipt_detail_page(props: &ReceiptDetailPageProps) -> Html {
    let (store, dispatch) = use_store::<AppStore>();
    let navigator = use_navigator().expect("ReceiptDetailPage must be rendered inside a Router");
    let receipt_id = props.receipt_id.clone();
    let trip_id = props.trip_id.clone();

    let photo = use_state(|| Option::<String>::None);

    // Load photo on mount
    {
        let receipt_id = receipt_id.clone();
        let db = store.db.clone();
        let photo = photo.clone();
        use_effect_with(receipt_id.clone(), move |_| {
            if let Some(db) = db {
                spawn_local(async move {
                    if let Ok(Some(data)) = get_photo(&db, &receipt_id).await {
                        photo.set(Some(data));
                    }
                });
            }
            || ()
        });
    }

    let receipt = store
        .current_receipts
        .iter()
        .find(|r| r.id == receipt_id)
        .cloned();

    let category = receipt.as_ref().and_then(|r| {
        store
            .categories
            .iter()
            .find(|c| c.id == r.category_id)
            .cloned()
    });

    let on_back = {
        let nav = navigator.clone();
        let tid = trip_id.clone();
        Callback::from(move |_| nav.push(&Route::TripDetail { id: tid.clone() }))
    };

    let on_edit = {
        let navigator = navigator.clone();
        let trip_id = trip_id.clone();
        let receipt_id = receipt_id.clone();
        Callback::from(move |_: MouseEvent| {
            navigator.push(&Route::EditReceipt {
                id: trip_id.clone(),
                rid: receipt_id.clone(),
            });
        })
    };

    let on_delete = {
        let store = store.clone();
        let dispatch = dispatch.clone();
        let navigator = navigator.clone();
        let receipt_id = receipt_id.clone();
        let trip_id = trip_id.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(db) = &store.db {
                let db = Rc::clone(db);
                let dispatch = dispatch.clone();
                let navigator = navigator.clone();
                let receipt_id = receipt_id.clone();
                let trip_id = trip_id.clone();
                spawn_local(async move {
                    let _ = delete_photo(&db, &receipt_id).await;
                    match delete_receipt(&db, &receipt_id).await {
                        Ok(()) => {
                            dispatch.reduce_mut(|s| {
                                s.current_receipts.retain(|r| r.id != receipt_id);
                            });
                            navigator.push(&Route::TripDetail { id: trip_id });
                        }
                        Err(e) => {
                            dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                        }
                    }
                });
            }
        })
    };

    let actions = html! {
        <>
            <button class="icon-btn" onclick={on_edit} title="Edit Receipt">
                <span class="material-icons">{"edit"}</span>
            </button>
            <button class="icon-btn danger" onclick={on_delete} title="Delete Receipt">
                <span class="material-icons">{"delete"}</span>
            </button>
        </>
    };

    html! {
        <div class="page">
            <AppBar title="Receipt" on_back={on_back} actions={actions} />
            <main class="page-content">
                if let Some(receipt) = &receipt {
                    <div class="receipt-detail">
                        <div class="detail-amount">
                            { format!("{} {:.2}", receipt.currency, receipt.amount) }
                        </div>

                        <div class="detail-row">
                            <span class="material-icons">{"category"}</span>
                            <span>{ category.as_ref().map(|c| c.name.as_str()).unwrap_or("Other") }</span>
                        </div>

                        <div class="detail-row">
                            <span class="material-icons">{"calendar_today"}</span>
                            <span>{ receipt.date.to_string() }</span>
                        </div>

                        if let Some(notes) = &receipt.notes {
                            <div class="detail-row">
                                <span class="material-icons">{"notes"}</span>
                                <span>{ notes }</span>
                            </div>
                        }

                        if let Some(photo_data) = &*photo {
                            <PhotoViewer data_uri={photo_data.clone()} />
                        }
                    </div>
                } else {
                    <div class="empty-state">
                        <p>{"Receipt not found"}</p>
                    </div>
                }
            </main>
        </div>
    }
}
