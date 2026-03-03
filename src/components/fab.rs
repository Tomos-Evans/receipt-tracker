use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FabProps {
    pub icon: String,
    pub label: String,
    pub onclick: Callback<MouseEvent>,
}

#[function_component(Fab)]
pub fn fab(props: &FabProps) -> Html {
    html! {
        <button
            class="fab"
            onclick={props.onclick.clone()}
            aria-label={props.label.clone()}
        >
            <span class="material-icons">{ &props.icon }</span>
        </button>
    }
}
