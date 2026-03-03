use yew::prelude::*;
use yew_router::prelude::*;
use crate::app::Route;

#[derive(Properties, PartialEq)]
pub struct AppBarProps {
    pub title: String,
    #[prop_or_default]
    pub show_back: bool,
    #[prop_or_default]
    pub actions: Html,
}

#[function_component(AppBar)]
pub fn app_bar(props: &AppBarProps) -> Html {
    let navigator = use_navigator().unwrap();

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.back())
    };

    html! {
        <header class="app-bar">
            <div class="app-bar-left">
                if props.show_back {
                    <button class="icon-btn" onclick={on_back} aria-label="Back">
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
