use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PhotoViewerProps {
    pub data_uri: String,
}

#[function_component(PhotoViewer)]
pub fn photo_viewer(props: &PhotoViewerProps) -> Html {
    html! {
        <div class="photo-viewer">
            <img
                src={props.data_uri.clone()}
                alt="Receipt photo"
                class="photo-viewer-img"
            />
        </div>
    }
}
