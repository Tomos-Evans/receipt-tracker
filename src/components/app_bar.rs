use crate::app::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AppBarProps {
    pub title: String,
    /// Explicit destination for the back button. None = no back button (show logo instead).
    #[prop_or_default]
    pub on_back: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub actions: Html,
}

#[function_component(AppBar)]
pub fn app_bar(props: &AppBarProps) -> Html {
    let navigator = use_navigator().unwrap();

    html! {
        <header class="app-bar">
            <div class="app-bar-left">
                if let Some(on_back) = &props.on_back {
                    <button class="icon-btn" onclick={on_back.clone()} aria-label="Back">
                        <span class="material-icons">{"arrow_back"}</span>
                    </button>
                } else {
                    <span class="material-icons app-logo">{"receipt_long"}</span>
                }
                <h1 class="app-bar-title">{ &props.title }</h1>
            </div>
            <div class="app-bar-actions">
                { props.actions.clone() }
                <button class="icon-btn" onclick={Callback::from(move |_| { navigator.push(&Route::Settings); })} aria-label="Settings">
                    <span class="material-icons">{"settings"}</span>
                </button>
            </div>
        </header>
    }
}
