use yew::prelude::*;

const CURRENCIES: &[(&str, &str)] = &[
    ("USD", "US Dollar"),
    ("EUR", "Euro"),
    ("GBP", "British Pound"),
    ("JPY", "Japanese Yen"),
    ("CAD", "Canadian Dollar"),
    ("AUD", "Australian Dollar"),
    ("CHF", "Swiss Franc"),
    ("CNY", "Chinese Yuan"),
    ("INR", "Indian Rupee"),
    ("MXN", "Mexican Peso"),
    ("BRL", "Brazilian Real"),
    ("KRW", "South Korean Won"),
    ("SGD", "Singapore Dollar"),
    ("HKD", "Hong Kong Dollar"),
    ("NOK", "Norwegian Krone"),
    ("SEK", "Swedish Krona"),
    ("DKK", "Danish Krone"),
    ("NZD", "New Zealand Dollar"),
    ("ZAR", "South African Rand"),
    ("THB", "Thai Baht"),
];

#[derive(Properties, PartialEq)]
pub struct CurrencySelectorProps {
    pub value: String,
    pub onchange: Callback<String>,
}

#[function_component(CurrencySelector)]
pub fn currency_selector(props: &CurrencySelectorProps) -> Html {
    let onchange = {
        let cb = props.onchange.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            cb.emit(select.value());
        })
    };

    html! {
        <div class="form-field">
            <label class="form-label">{"Currency"}</label>
            <select class="form-select" onchange={onchange} value={props.value.clone()}>
                { for CURRENCIES.iter().map(|(code, name)| {
                    let selected = *code == props.value;
                    html! {
                        <option value={*code} selected={selected}>
                            { format!("{} — {}", code, name) }
                        </option>
                    }
                })}
            </select>
        </div>
    }
}
