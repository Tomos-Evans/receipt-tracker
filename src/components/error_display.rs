use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorDisplayProps {
    pub message: String,
}

#[function_component(ErrorDisplay)]
pub fn error_display(props: &ErrorDisplayProps) -> Html {
    html! {
        <div class="error-display">
            <span class="material-icons error-icon">{"error_outline"}</span>
            <p>{ &props.message }</p>
        </div>
    }
}
