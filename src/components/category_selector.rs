use yew::prelude::*;
use crate::models::Category;

#[derive(Properties, PartialEq)]
pub struct CategorySelectorProps {
    pub categories: Vec<Category>,
    pub selected_id: String,
    pub onchange: Callback<String>,
}

#[function_component(CategorySelector)]
pub fn category_selector(props: &CategorySelectorProps) -> Html {
    let onchange = {
        let cb = props.onchange.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            cb.emit(select.value());
        })
    };

    html! {
        <div class="form-field">
            <label class="form-label">{"Category"}</label>
            <select class="form-select" onchange={onchange}>
                { for props.categories.iter().map(|cat| {
                    let selected = cat.id == props.selected_id;
                    html! {
                        <option value={cat.id.clone()} selected={selected}>
                            { &cat.name }
                        </option>
                    }
                })}
            </select>
        </div>
    }
}
