use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::app::Route;
use crate::components::app_bar::AppBar;
use crate::components::receipt_form::{ReceiptForm, ReceiptFormData};
use crate::state::AppStore;
use crate::storage::photos::{get_photo, save_photo, delete_photo};
use crate::storage::receipts::save_receipt;

#[derive(Properties, PartialEq)]
pub struct EditReceiptPageProps {
    pub trip_id: String,
    pub receipt_id: String,
}

#[function_component(EditReceiptPage)]
pub fn edit_receipt_page(props: &EditReceiptPageProps) -> Html {
    let (store, dispatch) = use_store::<AppStore>();
    let navigator = use_navigator().unwrap();
    let receipt_id = props.receipt_id.clone();
    let trip_id = props.trip_id.clone();

    let on_back = {
        let nav = navigator.clone();
        let tid = trip_id.clone();
        let rid = receipt_id.clone();
        Callback::from(move |_| nav.push(&Route::ReceiptDetail { id: tid.clone(), rid: rid.clone() }))
    };

    // form_data starts as None until the receipt + photo are loaded
    let form_data: UseStateHandle<Option<ReceiptFormData>> = use_state(|| None);

    // Load the existing receipt and photo once on mount
    {
        let receipt_id = receipt_id.clone();
        let db = store.db.clone();
        let form_data = form_data.clone();
        let receipt = store.current_receipts.iter()
            .find(|r| r.id == receipt_id)
            .cloned();

        use_effect_with(receipt_id.clone(), move |_| {
            if let Some(receipt) = receipt {
                if let Some(db) = db {
                    spawn_local(async move {
                        let photo = get_photo(&db, &receipt.id).await.ok().flatten();
                        form_data.set(Some(ReceiptFormData::from_receipt(&receipt, photo)));
                    });
                } else {
                    form_data.set(Some(ReceiptFormData::from_receipt(&receipt, None)));
                }
            }
            || ()
        });
    }

    let on_change = {
        let form_data = form_data.clone();
        Callback::from(move |data: ReceiptFormData| form_data.set(Some(data)))
    };

    let on_submit = {
        let store = store.clone();
        let dispatch = dispatch.clone();
        let navigator = navigator.clone();
        let receipt_id = receipt_id.clone();
        let trip_id = trip_id.clone();
        Callback::from(move |data: ReceiptFormData| {
            let db = match store.db.as_ref() {
                Some(db) => Rc::clone(db),
                None => return,
            };
            // Find the existing receipt to preserve id, trip_id, created_at
            let existing = match store.current_receipts.iter().find(|r| r.id == receipt_id) {
                Some(r) => r.clone(),
                None => return,
            };

            let mut updated = existing;
            updated.amount = data.amount_f64().unwrap_or(updated.amount);
            updated.category_id = data.category_id.clone();
            updated.notes = if data.notes.is_empty() { None } else { Some(data.notes.clone()) };
            updated.date = data.date_naive().unwrap_or(updated.date);

            let photo = data.photo.clone();
            let dispatch = dispatch.clone();
            let navigator = navigator.clone();
            let trip_id = trip_id.clone();

            spawn_local(async move {
                // Upsert receipt
                if let Err(e) = save_receipt(&db, &updated).await {
                    dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                    return;
                }

                // Update photo: save new data or delete if cleared
                let photo_result = match photo {
                    Some(data) => save_photo(&db, &updated.id, data).await,
                    None => delete_photo(&db, &updated.id).await,
                };
                if let Err(e) = photo_result {
                    dispatch.reduce_mut(|s| s.error = Some(e.to_string()));
                    return;
                }

                // Update store in-place
                let updated_clone = updated.clone();
                dispatch.reduce_mut(|s| {
                    if let Some(r) = s.current_receipts.iter_mut().find(|r| r.id == updated_clone.id) {
                        *r = updated_clone;
                    }
                });

                navigator.push(&Route::ReceiptDetail {
                    id: trip_id,
                    rid: updated.id,
                });
            });
        })
    };

    html! {
        <div class="page">
            <AppBar title="Edit Receipt" on_back={on_back} />
            <main class="page-content">
                if let Some(data) = (*form_data).clone() {
                    <ReceiptForm
                        data={data}
                        categories={store.categories.clone()}
                        on_change={on_change}
                        on_submit={on_submit}
                        submit_label="Save Changes"
                    />
                } else {
                    <div class="loading-screen">
                        <div class="spinner" />
                    </div>
                }
            </main>
        </div>
    }
}
