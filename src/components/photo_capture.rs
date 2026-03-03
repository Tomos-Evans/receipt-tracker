use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlCanvasElement, HtmlVideoElement};
use yew::prelude::*;

/// Max width in pixels to resize images before storage
const MAX_WIDTH: f64 = 800.0;
const JPEG_QUALITY: f64 = 0.7;

#[derive(Properties, PartialEq)]
pub struct PhotoCaptureProps {
    pub on_photo: Callback<String>,
    pub current_photo: Option<String>,
}

pub enum Msg {
    FileSelected,
    ClearPhoto,
    CameraCapture,
    GotStream(web_sys::MediaStream),
    SnapPhoto,
    CameraError(String),
    CloseCamera,
}

pub struct PhotoCapture {
    file_input_ref: NodeRef,
    canvas_ref: NodeRef,
    video_ref: NodeRef,
    camera_open: bool,
    camera_error: Option<String>,
    stream: Option<web_sys::MediaStream>,
}

impl Component for PhotoCapture {
    type Message = Msg;
    type Properties = PhotoCaptureProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            file_input_ref: NodeRef::default(),
            canvas_ref: NodeRef::default(),
            video_ref: NodeRef::default(),
            camera_open: false,
            camera_error: None,
            stream: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileSelected => {
                let input = match self.file_input_ref.cast::<HtmlInputElement>() {
                    Some(i) => i,
                    None => return false,
                };
                let canvas = match self.canvas_ref.cast::<HtmlCanvasElement>() {
                    Some(c) => c,
                    None => return false,
                };
                let on_photo = ctx.props().on_photo.clone();

                if let Some(files) = input.files()
                    && let Some(file) = files.get(0)
                {
                    let file = gloo_file::File::from(file);
                    spawn_local(async move {
                        if let Ok(data) = gloo_file::futures::read_as_data_url(&file).await {
                            // Attempt resize; fall back to original on error
                            let result = resize_image_data_url(&data, &canvas).await;
                            on_photo.emit(result.unwrap_or(data));
                        }
                    });
                }
                false
            }

            Msg::ClearPhoto => {
                if let Some(input) = self.file_input_ref.cast::<HtmlInputElement>() {
                    input.set_value("");
                }
                ctx.props().on_photo.emit(String::new());
                false
            }

            Msg::CameraCapture => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    let window = match web_sys::window() {
                        Some(w) => w,
                        None => { link.send_message(Msg::CameraError("No window".into())); return; }
                    };
                    let navigator = window.navigator();
                    match navigator.media_devices() {
                        Ok(devices) => {
                            let constraints = web_sys::MediaStreamConstraints::new();
                            constraints.set_video(&JsValue::from_bool(true));
                            let promise = match devices.get_user_media_with_constraints(&constraints) {
                                Ok(p) => p,
                                Err(e) => {
                                    link.send_message(Msg::CameraError(format!("{:?}", e)));
                                    return;
                                }
                            };
                            match wasm_bindgen_futures::JsFuture::from(promise).await {
                                Ok(stream_val) => {
                                    let stream: web_sys::MediaStream = stream_val.unchecked_into();
                                    link.send_message(Msg::GotStream(stream));
                                }
                                Err(e) => {
                                    link.send_message(Msg::CameraError(format!("Camera denied: {:?}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            link.send_message(Msg::CameraError(format!("No media devices: {:?}", e)));
                        }
                    }
                });
                false
            }

            Msg::GotStream(stream) => {
                // Attach stream to video element
                if let Some(video) = self.video_ref.cast::<HtmlVideoElement>() {
                    video.set_src_object(Some(&stream));
                }
                self.stream = Some(stream);
                self.camera_open = true;
                true
            }

            Msg::SnapPhoto => {
                let canvas = match self.canvas_ref.cast::<HtmlCanvasElement>() {
                    Some(c) => c,
                    None => return false,
                };
                let video = match self.video_ref.cast::<HtmlVideoElement>() {
                    Some(v) => v,
                    None => return false,
                };
                let on_photo = ctx.props().on_photo.clone();

                let w = video.video_width() as f64;
                let h = video.video_height() as f64;
                let (cw, ch) = if w > MAX_WIDTH {
                    (MAX_WIDTH, h * MAX_WIDTH / w)
                } else {
                    (w, h)
                };
                canvas.set_width(cw as u32);
                canvas.set_height(ch as u32);

                if let Ok(Some(obj)) = canvas.get_context("2d") {
                    let ctx2d: web_sys::CanvasRenderingContext2d = obj.unchecked_into();
                    let _ = ctx2d.draw_image_with_html_video_element_and_dw_and_dh(
                        &video, 0.0, 0.0, cw, ch
                    );
                }

                let data_url = canvas
                    .to_data_url_with_type_and_encoder_options(
                        "image/jpeg",
                        &JsValue::from_f64(JPEG_QUALITY),
                    )
                    .unwrap_or_default();

                // Stop camera stream
                if let Some(stream) = &self.stream {
                    let tracks = stream.get_tracks();
                    for i in 0..tracks.length() {
                        let track: web_sys::MediaStreamTrack = tracks.get(i).unchecked_into();
                        track.stop();
                    }
                }
                self.stream = None;
                self.camera_open = false;

                on_photo.emit(data_url);
                true
            }

            Msg::CameraError(e) => {
                self.camera_error = Some(e);
                true
            }

            Msg::CloseCamera => {
                self.camera_open = false;
                self.camera_error = None;
                if let Some(stream) = self.stream.take() {
                    let tracks = stream.get_tracks();
                    for i in 0..tracks.length() {
                        let track: web_sys::MediaStreamTrack = tracks.get(i).unchecked_into();
                        track.stop();
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_file_change = ctx.link().callback(|_: Event| Msg::FileSelected);
        let on_camera = ctx.link().callback(|_: MouseEvent| Msg::CameraCapture);
        let on_snap = ctx.link().callback(|_: MouseEvent| Msg::SnapPhoto);
        let on_close = ctx.link().callback(|_: MouseEvent| Msg::CloseCamera);
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

                if self.camera_open {
                    <div class="camera-overlay">
                        <video ref={self.video_ref.clone()} class="camera-video" autoplay=true playsinline=true />
                        <div class="camera-controls">
                            <button class="btn btn-primary" onclick={on_snap}>
                                <span class="material-icons">{"camera"}</span>
                                {" Capture"}
                            </button>
                            <button class="btn btn-secondary" onclick={on_close}>{"Cancel"}</button>
                        </div>
                    </div>
                }

                if let Some(err) = &self.camera_error {
                    <p class="form-error">{ err }</p>
                }

                <div class="photo-actions">
                    <label class="btn btn-secondary photo-pick-btn">
                        <span class="material-icons">{"photo_library"}</span>
                        {" Gallery / Camera"}
                        <input
                            ref={self.file_input_ref.clone()}
                            type="file"
                            accept="image/*"
                            capture="environment"
                            style="display:none"
                            onchange={on_file_change}
                        />
                    </label>
                    <button class="btn btn-secondary" onclick={on_camera}>
                        <span class="material-icons">{"videocam"}</span>
                        {" Live Camera"}
                    </button>
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
        .to_data_url_with_type_and_encoder_options(
            "image/jpeg",
            &JsValue::from_f64(JPEG_QUALITY),
        )
        .map_err(|e| format!("{:?}", e))
}
