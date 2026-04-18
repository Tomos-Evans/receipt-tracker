use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, HtmlInputElement};
use yew::prelude::*;

use super::photo_editor::PhotoEditor;

/// Max width in pixels to resize images before storage
const MAX_WIDTH: f64 = 800.0;
const JPEG_QUALITY: f64 = 0.7;

#[derive(Properties, PartialEq)]
pub struct PhotoCaptureProps {
    pub on_photo: Callback<String>,
    pub current_photo: Option<String>,
}

pub enum Msg {
    FileSelected(NodeRef),
    ClearPhoto,
    StartEdit(String),
    PhotoEdited(String),
    EditCancelled,
}

pub struct PhotoCapture {
    gallery_input_ref: NodeRef,
    camera_input_ref: NodeRef,
    canvas_ref: NodeRef,
    pending_edit: Option<String>,
}

impl Component for PhotoCapture {
    type Message = Msg;
    type Properties = PhotoCaptureProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            gallery_input_ref: NodeRef::default(),
            camera_input_ref: NodeRef::default(),
            canvas_ref: NodeRef::default(),
            pending_edit: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileSelected(input_ref) => {
                let input = match input_ref.cast::<HtmlInputElement>() {
                    Some(i) => i,
                    None => return false,
                };
                let canvas = match self.canvas_ref.cast::<HtmlCanvasElement>() {
                    Some(c) => c,
                    None => return false,
                };
                let link = ctx.link().clone();

                if let Some(files) = input.files()
                    && let Some(file) = files.get(0)
                {
                    let file = gloo_file::File::from(file);
                    spawn_local(async move {
                        if let Ok(data) = gloo_file::futures::read_as_data_url(&file).await {
                            let result = resize_image_data_url(&data, &canvas).await;
                            link.send_message(Msg::StartEdit(result.unwrap_or(data)));
                        }
                    });
                }
                false
            }

            Msg::ClearPhoto => {
                if let Some(input) = self.gallery_input_ref.cast::<HtmlInputElement>() {
                    input.set_value("");
                }
                if let Some(input) = self.camera_input_ref.cast::<HtmlInputElement>() {
                    input.set_value("");
                }
                ctx.props().on_photo.emit(String::new());
                false
            }

            Msg::StartEdit(url) => {
                self.pending_edit = Some(url);
                true
            }

            Msg::PhotoEdited(url) => {
                self.pending_edit = None;
                ctx.props().on_photo.emit(url);
                false
            }

            Msg::EditCancelled => {
                self.pending_edit = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Show editor when a photo is pending edit
        if let Some(src) = &self.pending_edit {
            return html! {
                <PhotoEditor
                    src={src.clone()}
                    on_done={ctx.link().callback(Msg::PhotoEdited)}
                    on_cancel={ctx.link().callback(|_| Msg::EditCancelled)}
                />
            };
        }

        let gallery_ref = self.gallery_input_ref.clone();
        let on_gallery_change = ctx
            .link()
            .callback(move |_: Event| Msg::FileSelected(gallery_ref.clone()));

        let camera_ref = self.camera_input_ref.clone();
        let on_camera_change = ctx
            .link()
            .callback(move |_: Event| Msg::FileSelected(camera_ref.clone()));

        let on_clear = ctx.link().callback(|_: MouseEvent| Msg::ClearPhoto);

        html! {
            <div class="photo-capture">
                // Hidden canvas for image processing
                <canvas ref={self.canvas_ref.clone()} style="display:none" />

                if let Some(photo) = &ctx.props().current_photo {
                    if !photo.is_empty() {
                        <div class="photo-preview-wrapper">
                            <img src={photo.clone()} class="photo-preview" alt="Receipt photo" />
                            <button class="btn-icon photo-clear-btn" onclick={on_clear}>
                                <span class="material-icons">{"close"}</span>
                            </button>
                        </div>
                    }
                }

                <div class="photo-actions">
                    <label class="btn btn-secondary photo-pick-btn">
                        <span class="material-icons">{"photo_library"}</span>
                        {" Gallery"}
                        <input
                            ref={self.gallery_input_ref.clone()}
                            type="file"
                            accept="image/*"
                            style="display:none"
                            onchange={on_gallery_change}
                        />
                    </label>
                    <label class="btn btn-secondary photo-pick-btn">
                        <span class="material-icons">{"camera_alt"}</span>
                        {" Camera"}
                        <input
                            ref={self.camera_input_ref.clone()}
                            type="file"
                            accept="image/*"
                            capture="environment"
                            style="display:none"
                            onchange={on_camera_change}
                        />
                    </label>
                </div>
            </div>
        }
    }
}

/// Resize an image data URL to max MAX_WIDTH using a canvas element.
async fn resize_image_data_url(
    data_url: &str,
    canvas: &HtmlCanvasElement,
) -> Result<String, String> {
    let document = web_sys::window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    let img: web_sys::HtmlImageElement = document
        .create_element("img")
        .map_err(|e| format!("{:?}", e))?
        .unchecked_into();

    // Use a Promise to wait for image load
    let img_clone = img.clone();
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let img_inner = img_clone.clone();
        let resolve_inner = resolve.clone();
        let onload = Closure::once(Box::new(move || {
            let _ = resolve_inner.call1(&JsValue::NULL, &JsValue::NULL);
            drop(img_inner);
        }) as Box<dyn FnOnce()>);
        img_clone.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
    });

    img.set_src(data_url);
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| format!("{:?}", e))?;

    let w = img.natural_width() as f64;
    let h = img.natural_height() as f64;
    let (cw, ch) = if w > MAX_WIDTH {
        (MAX_WIDTH, h * MAX_WIDTH / w)
    } else {
        (w, h)
    };

    canvas.set_width(cw as u32);
    canvas.set_height(ch as u32);

    if let Ok(Some(obj)) = canvas.get_context("2d") {
        let ctx2d: web_sys::CanvasRenderingContext2d = obj.unchecked_into();
        ctx2d
            .draw_image_with_html_image_element_and_dw_and_dh(&img, 0.0, 0.0, cw, ch)
            .map_err(|e| format!("{:?}", e))?;
    }

    canvas
        .to_data_url_with_type_and_encoder_options("image/jpeg", &JsValue::from_f64(JPEG_QUALITY))
        .map_err(|e| format!("{:?}", e))
}
