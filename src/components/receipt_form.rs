use crate::components::category_selector::CategorySelector;
use crate::components::currency_selector::CurrencySelector;
use crate::components::photo_capture::PhotoCapture;
use crate::models::Category;
use chrono::NaiveDate;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ReceiptFormData {
    pub amount: String,
    pub category_id: String,
    pub notes: String,
    pub date: String,
    pub photo: Option<String>,
    pub currency: String,
}

impl ReceiptFormData {
    pub fn new(default_category_id: String, default_currency: String) -> Self {
        Self {
            amount: String::new(),
            category_id: default_category_id,
            notes: String::new(),
            date: chrono::Local::now().date_naive().to_string(),
            photo: None,
            currency: default_currency,
        }
    }

    pub fn from_receipt(receipt: &crate::models::Receipt, photo: Option<String>) -> Self {
        Self {
            amount: format!("{:.2}", receipt.amount),
            category_id: receipt.category_id.clone(),
            notes: receipt.notes.clone().unwrap_or_default(),
            date: receipt.date.to_string(),
            photo,
            currency: receipt.currency.clone(),
        }
    }

    pub fn amount_f64(&self) -> Option<f64> {
        self.amount.parse::<f64>().ok().filter(|&v| v >= 0.0)
    }

    pub fn date_naive(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").ok()
    }

    pub fn is_valid(&self) -> bool {
        self.amount_f64().is_some() && self.date_naive().is_some() && !self.category_id.is_empty()
    }
}

fn compute_claimable_with_tax(total: &str, subtotal: &str, claimable: &str) -> Option<f64> {
    let t: f64 = total.parse().ok().filter(|&v: &f64| v > 0.0)?;
    let s: f64 = subtotal.parse().ok().filter(|&v: &f64| v > 0.0)?;
    let c: f64 = claimable.parse().ok().filter(|&v: &f64| v >= 0.0)?;
    Some(c * (1.0 + (t - s) / s))
}

fn compute_tax_pct(total: &str, subtotal: &str) -> Option<f64> {
    let t: f64 = total.parse().ok().filter(|&v: &f64| v > 0.0)?;
    let s: f64 = subtotal.parse().ok().filter(|&v: &f64| v > 0.0)?;
    Some((t - s) / s * 100.0)
}

#[derive(Clone, PartialEq)]
enum FormTab {
    Basic,
    Advanced,
}

#[derive(Properties, PartialEq)]
pub struct ReceiptFormProps {
    pub data: ReceiptFormData,
    pub categories: Vec<Category>,
    pub on_change: Callback<ReceiptFormData>,
    pub on_submit: Callback<ReceiptFormData>,
    pub submit_label: String,
}

#[function_component(ReceiptForm)]
pub fn receipt_form(props: &ReceiptFormProps) -> Html {
    let data = props.data.clone();
    let active_tab = use_state(|| FormTab::Basic);
    let adv_total = use_state(String::new);
    let adv_subtotal = use_state(String::new);
    let adv_claimable = use_state(String::new);

    let on_tab_basic = {
        let active_tab = active_tab.clone();
        Callback::from(move |_: MouseEvent| active_tab.set(FormTab::Basic))
    };

    let on_tab_advanced = {
        let active_tab = active_tab.clone();
        Callback::from(move |_: MouseEvent| active_tab.set(FormTab::Advanced))
    };

    let on_amount = {
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut d = data.clone();
            d.amount = input.value();
            cb.emit(d);
        })
    };

    let on_adv_total = {
        let adv_total = adv_total.clone();
        let adv_subtotal = adv_subtotal.clone();
        let adv_claimable = adv_claimable.clone();
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let val = input.value();
            adv_total.set(val.clone());
            if let Some(result) = compute_claimable_with_tax(&val, &adv_subtotal, &adv_claimable) {
                let mut d = data.clone();
                d.amount = format!("{:.2}", result);
                cb.emit(d);
            }
        })
    };

    let on_adv_subtotal = {
        let adv_total = adv_total.clone();
        let adv_subtotal = adv_subtotal.clone();
        let adv_claimable = adv_claimable.clone();
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let val = input.value();
            adv_subtotal.set(val.clone());
            if let Some(result) = compute_claimable_with_tax(&adv_total, &val, &adv_claimable) {
                let mut d = data.clone();
                d.amount = format!("{:.2}", result);
                cb.emit(d);
            }
        })
    };

    let on_adv_claimable = {
        let adv_total = adv_total.clone();
        let adv_subtotal = adv_subtotal.clone();
        let adv_claimable = adv_claimable.clone();
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let val = input.value();
            adv_claimable.set(val.clone());
            if let Some(result) = compute_claimable_with_tax(&adv_total, &adv_subtotal, &val) {
                let mut d = data.clone();
                d.amount = format!("{:.2}", result);
                cb.emit(d);
            }
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

    let on_category = {
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |id: String| {
            let mut d = data.clone();
            d.category_id = id;
            cb.emit(d);
        })
    };

    let on_notes = {
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let ta: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let mut d = data.clone();
            d.notes = ta.value();
            cb.emit(d);
        })
    };

    let on_date = {
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut d = data.clone();
            d.date = input.value();
            cb.emit(d);
        })
    };

    let on_photo = {
        let data = data.clone();
        let cb = props.on_change.clone();
        Callback::from(move |photo: String| {
            let mut d = data.clone();
            d.photo = if photo.is_empty() { None } else { Some(photo) };
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

    let tax_pct = compute_tax_pct(&adv_total, &adv_subtotal);
    let claimable_with_tax = compute_claimable_with_tax(&adv_total, &adv_subtotal, &adv_claimable);

    html! {
        <form class="form" onsubmit={on_submit}>
            <div class="form-tabs">
                <button
                    type="button"
                    class={if *active_tab == FormTab::Basic { "form-tab active" } else { "form-tab" }}
                    onclick={on_tab_basic}
                >{"Basic"}</button>
                <button
                    type="button"
                    class={if *active_tab == FormTab::Advanced { "form-tab active" } else { "form-tab" }}
                    onclick={on_tab_advanced}
                >{"Advanced"}</button>
            </div>

            if *active_tab == FormTab::Basic {
                <div class="form-field">
                    <label class="form-label">{"Amount"}</label>
                    <input
                        class="form-input"
                        type="number"
                        placeholder="0.00"
                        min="0"
                        step="0.01"
                        value={data.amount.clone()}
                        oninput={on_amount}
                        required=true
                    />
                </div>
            } else {
                <div class="form-field">
                    <label class="form-label">{"Total receipt amount"}</label>
                    <input
                        class="form-input"
                        type="number"
                        placeholder="0.00"
                        min="0"
                        step="0.01"
                        value={(*adv_total).clone()}
                        oninput={on_adv_total}
                    />
                </div>
                <div class="form-field">
                    <label class="form-label">{"Subtotal before tax & tip"}</label>
                    <input
                        class="form-input"
                        type="number"
                        placeholder="0.00"
                        min="0"
                        step="0.01"
                        value={(*adv_subtotal).clone()}
                        oninput={on_adv_subtotal}
                    />
                    if let Some(pct) = tax_pct {
                        <span class="adv-hint">{format!("Tax/tip overhead: {:.1}%", pct)}</span>
                    }
                </div>
                <div class="form-field">
                    <label class="form-label">{"Claimable before tax & tip"}</label>
                    <input
                        class="form-input"
                        type="number"
                        placeholder="0.00"
                        min="0"
                        step="0.01"
                        value={(*adv_claimable).clone()}
                        oninput={on_adv_claimable}
                    />
                </div>
                if let Some(total) = claimable_with_tax {
                    <div class="adv-result">
                        <span class="adv-result-label">{"Claimable amount (inc. tax/tip)"}</span>
                        <span class="adv-result-value">{format!("{:.2}", total)}</span>
                    </div>
                }
            }

            <CurrencySelector
                value={data.currency.clone()}
                onchange={on_currency}
            />

            <CategorySelector
                categories={props.categories.clone()}
                selected_id={data.category_id.clone()}
                onchange={on_category}
            />

            <div class="form-field">
                <label class="form-label">{"Date"}</label>
                <input
                    class="form-input"
                    type="date"
                    value={data.date.clone()}
                    oninput={on_date}
                    required=true
                />
            </div>

            <div class="form-field">
                <label class="form-label">{"Notes (optional)"}</label>
                <textarea
                    class="form-textarea"
                    placeholder="What was this for?"
                    value={data.notes.clone()}
                    oninput={on_notes}
                    rows="3"
                />
            </div>

            <div class="form-field">
                <label class="form-label">{"Photo (optional)"}</label>
                <PhotoCapture
                    on_photo={on_photo}
                    current_photo={data.photo.clone()}
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
