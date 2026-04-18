use crate::app::Route;
use crate::models::Trip;
use std::collections::BTreeMap;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TripCardProps {
    pub trip: Trip,
    pub receipt_count: usize,
    pub currency_totals: BTreeMap<String, f64>,
}

#[function_component(TripCard)]
pub fn trip_card(props: &TripCardProps) -> Html {
    let navigator = use_navigator().expect("TripCard must be rendered inside a Router");
    let trip_id = props.trip.id.clone();

    let onclick = Callback::from(move |_| {
        navigator.push(&Route::TripDetail {
            id: trip_id.clone(),
        });
    });

    let total_str: String = if props.currency_totals.is_empty() {
        format!("{} 0.00", props.trip.currency)
    } else {
        props
            .currency_totals
            .iter()
            .map(|(c, a)| format!("{} {:.2}", c, a))
            .collect::<Vec<_>>()
            .join(" · ")
    };

    html! {
        <div class="card trip-card" onclick={onclick}>
            <div class="card-content">
                <div class="trip-card-header">
                    <span class="material-icons trip-icon">{"luggage"}</span>
                    <div class="trip-info">
                        <h3 class="trip-name">{ &props.trip.name }</h3>
                        <p class="trip-dates">
                            { format!("{} – {}", props.trip.start_date, props.trip.end_date) }
                        </p>
                    </div>
                </div>
                <div class="trip-card-footer">
                    <span class="trip-receipts">
                        <span class="material-icons small">{"receipt"}</span>
                        { format!("{} receipts", props.receipt_count) }
                    </span>
                    <span class="trip-total">
                        { total_str }
                    </span>
                </div>
            </div>
        </div>
    }
}
