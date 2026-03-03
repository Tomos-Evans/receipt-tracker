use chrono::NaiveDate;
use yew::prelude::*;
use crate::components::currency_selector::CurrencySelector;

#[derive(Clone, PartialEq)]
pub struct TripFormData {
    pub name: String,
    pub currency: String,
    pub start_date: String,
    pub end_date: String,
}

impl Default for TripFormData {
    fn default() -> Self {
        let today = chrono::Local::now().date_naive().to_string();
        Self {
            name: String::new(),
            currency: "USD".to_string(),
            start_date: today.clone(),
            end_date: today,
        }
    }
}

impl TripFormData {
    pub fn start_naive(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d").ok()
    }
    pub fn end_naive(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.end_date, "%Y-%m-%d").ok()
    }
    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && self.start_naive().is_some()
            && self.end_naive().is_some()
            && self.start_naive() <= self.end_naive()
    }
}

#[derive(Properties, PartialEq)]
pub struct TripFormProps {
    pub data: TripFormData,
    pub on_change: Callback<TripFormData>,
    pub on_submit: Callback<TripFormData>,
    pub submit_label: String,
}

#[function_component(TripForm)]
pub fn trip_form(props: &TripFormProps) -> Html {
    let data = props.data.clone();

    let on_name = {
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut d = data.clone();
            d.name = input.value();
            cb.emit(d);
        })
    };

    let on_currency = {
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |val: String| {
            let mut d = data.clone();
            d.currency = val;
            cb.emit(d);
        })
    };

    let on_start = {
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut d = data.clone();
            d.start_date = input.value();
            cb.emit(d);
        })
    };

    let on_end = {
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut d = data.clone();
            d.end_date = input.value();
            cb.emit(d);
        })
    };

    let on_submit = {
        let data = data.clone();
        let cb = props.on_submit.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if data.is_valid() {
                cb.emit(data.clone());
            }
        })
    };

    html! {
        <form class="form" onsubmit={on_submit}>
            <div class="form-field">
                <label class="form-label">{"Trip Name"}</label>
                <input
                    class="form-input"
                    type="text"
                    placeholder="e.g. Tokyo 2026"
                    value={data.name.clone()}
                    oninput={on_name}
                    required=true
                />
            </div>

            <CurrencySelector
                value={data.currency.clone()}
                onchange={on_currency}
            />

            <div class="form-field">
                <label class="form-label">{"Start Date"}</label>
                <input
                    class="form-input"
                    type="date"
                    value={data.start_date.clone()}
                    oninput={on_start}
                    required=true
                />
            </div>

            <div class="form-field">
                <label class="form-label">{"End Date"}</label>
                <input
                    class="form-input"
                    type="date"
                    value={data.end_date.clone()}
                    oninput={on_end}
                    required=true
                />
            </div>

            <button
                type="submit"
                class="btn btn-primary btn-full"
                disabled={!data.is_valid()}
            >
                { &props.submit_label }
            </button>
        </form>
    }
}
