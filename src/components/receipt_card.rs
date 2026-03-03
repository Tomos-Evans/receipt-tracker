use yew::prelude::*;
use yew_router::prelude::*;
use crate::app::Route;
use crate::models::{Receipt, Category};

#[derive(Properties, PartialEq)]
pub struct ReceiptCardProps {
    pub receipt: Receipt,
    pub category: Option<Category>,
    pub currency: String,
    pub has_photo: bool,
}

#[function_component(ReceiptCard)]
pub fn receipt_card(props: &ReceiptCardProps) -> Html {
    let navigator = use_navigator().unwrap();
    let trip_id = props.receipt.trip_id.clone();
    let receipt_id = props.receipt.id.clone();

    let onclick = Callback::from(move |_| {
        navigator.push(&Route::ReceiptDetail {
            id: trip_id.clone(),
            rid: receipt_id.clone(),
        });
    });

    let icon = props.category.as_ref()
        .and_then(|c| c.icon.as_deref())
        .unwrap_or("receipt");
    let color = props.category.as_ref()
        .and_then(|c| c.color.as_deref())
        .unwrap_or("#757575");
    let cat_name = props.category.as_ref()
        .map(|c| c.name.as_str())
        .unwrap_or("Other");

    html! {
        <div class="card receipt-card" onclick={onclick}>
            <div class="receipt-card-icon" style={format!("color:{}", color)}>
                <span class="material-icons">{ icon }</span>
            </div>
            <div class="receipt-card-body">
                <div class="receipt-card-top">
                    <span class="receipt-category">{ cat_name }</span>
                    if props.has_photo {
                        <span class="material-icons photo-indicator small">{"photo_camera"}</span>
                    }
                </div>
                <p class="receipt-notes">{ props.receipt.notes.as_deref().unwrap_or("") }</p>
                <p class="receipt-date small">{ props.receipt.date.to_string() }</p>
            </div>
            <div class="receipt-card-amount">
                { format!("{} {:.2}", props.currency, props.receipt.amount) }
            </div>
        </div>
    }
}
