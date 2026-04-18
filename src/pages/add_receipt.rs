use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::app::Route;
use crate::components::app_bar::AppBar;
use crate::components::receipt_form::{ReceiptForm, ReceiptFormData};
use crate::models::{Receipt, category::CAT_BREAKFAST_ID};
use crate::state::AppStore;
use crate::storage::photos::save_photo;
use crate::storage::receipts::save_receipt;

#[derive(Properties, PartialEq)]
pub struct AddReceiptPageProps {
    pub trip_id: String,
}

#[function_component(AddReceiptPage)]
pub fn add_receipt_page(props: &AddReceiptPageProps) -> Html {
    let (store, dispatch) = use_store::<AppStore>();
    let navigator = use_navigator().expect("AddReceiptPage must be rendered inside a Router");
    let trip_id = props.trip_id.clone();

    let on_back = {
        let nav = navigator.clone();
        let tid = trip_id.clone();
        Callback::from(move |_| nav.push(&Route::TripDetail { id: tid.clone() }))
    };

    let default_cat = store
        .categories
        .first()
        .map(|c| c.id.clone())
        .unwrap_or_else(|| CAT_BREAKFAST_ID.to_string());

    let trip_currency = store
        .trips
        .iter()
        .find(|t| t.id == trip_id)
        .map(|t| t.currency.clone())
        .unwrap_or_else(|| "USD".to_string());

    let form_data = use_state(move || ReceiptFormData::new(default_cat, trip_currency));

    let on_change = {
        let form_data = form_data.clone();
        Callback::from(move |data| form_data.set(data))
    };

    let on_submit = {
        let store = store.clone();
        let dispatch = dispatch.clone();
        let navigator = navigator.clone();
        let trip_id = trip_id.clone();
        Callback::from(move |data: ReceiptFormData| {
            let db = match &store.db {
                Some(db) => Rc::clone(db),
                None => return,
            };
            let dispatch = dispatch.clone();
            let navigator = navigator.clone();
            let trip_id = trip_id.clone();
            let photo = data.photo.clone();

            let Some(date) = data.date_naive() else {
                return;
            };
            let receipt = Receipt::new(
                trip_id.clone(),
                data.amount_f64().unwrap_or(0.0),
                data.category_id.clone(),
                if data.notes.is_empty() {
                    None
                } else {
                    Some(data.notes.clone())
                },
                date,
                data.currency.clone(),
            );

            spawn_local(async move {
                match save_receipt(&db, &receipt).await {
                    Ok(()) => {
                        // Save photo if present
                        if let Some(photo_data) = photo
                            && !photo_data.is_empty()
                        {
                            let _ = save_photo(&db, &receipt.id, photo_data).await;
                        }
                        let receipt_clone = receipt.clone();
                        dispatch.reduce_mut(|s| {
                            s.current_receipts.insert(0, receipt_clone);
                        });
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
            <AppBar title="New Receipt" on_back={on_back} />
            <main class="page-content">
                <ReceiptForm
                    data={(*form_data).clone()}
                    categories={store.categories.clone()}
                    on_change={on_change}
                    on_submit={on_submit}
                    submit_label="Save Receipt"
                />
            </main>
        </div>
    }
}
