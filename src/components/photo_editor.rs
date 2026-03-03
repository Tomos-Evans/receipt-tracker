use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, HtmlImageElement, PointerEvent, TouchEvent};
use yew::prelude::*;

const JPEG_QUALITY: f64 = 0.85;

#[derive(Properties, PartialEq)]
pub struct PhotoEditorProps {
    pub src: String,
    pub on_done: Callback<String>,
    pub on_cancel: Callback<()>,
}

#[derive(Clone, PartialEq)]
pub enum EditorMode {
    Idle,
    Draw,
    Crop,
}

pub enum Msg {
    /// New base image loaded into HtmlImageElement; draw to canvas
    Loaded,
    /// Pointer/touch down (canvas coordinates)
    InputDown(f64, f64),
    /// Pointer/touch move (canvas coordinates)
    InputMove(f64, f64),
    /// Pointer/touch up
    InputUp,
    SetMode(EditorMode),
    RotateCw,
    RotateCcw,
    ApplyCrop,
    /// Async rotate/crop result; reload image
    SetBase(String),
    UsePhoto,
    Cancel,
    Noop,
}

pub struct PhotoEditor {
    canvas_ref: NodeRef,
    base_data_url: String,
    loaded_img: Option<HtmlImageElement>,
    strokes: Vec<Vec<(f64, f64)>>,
    active_stroke: Option<Vec<(f64, f64)>>,
    mode: EditorMode,
    crop_start: Option<(f64, f64)>,
    crop_end: Option<(f64, f64)>,
}

impl PhotoEditor {
    fn canvas(&self) -> Option<HtmlCanvasElement> {
        self.canvas_ref.cast::<HtmlCanvasElement>()
    }

    /// Map client coordinates to canvas pixel coordinates
    fn client_to_canvas(canvas: &HtmlCanvasElement, cx: f64, cy: f64) -> (f64, f64) {
        let rect = canvas.get_bounding_client_rect();
        let sx = canvas.width() as f64 / rect.width();
        let sy = canvas.height() as f64 / rect.height();
        ((cx - rect.left()) * sx, (cy - rect.top()) * sy)
    }

    /// Draw base image + all completed strokes onto the canvas
    fn redraw_canvas(&self) {
        let canvas = match self.canvas() {
            Some(c) => c,
            None => return,
        };
        let img = match &self.loaded_img {
            Some(i) => i,
            None => return,
        };
        let ctx2d = match canvas.get_context("2d") {
            Ok(Some(obj)) => obj.unchecked_into::<web_sys::CanvasRenderingContext2d>(),
            _ => return,
        };

        // Draw base image
        let _ = ctx2d.draw_image_with_html_image_element(img, 0.0, 0.0);

        // Replay all completed strokes
        self.draw_strokes(&ctx2d, &self.strokes);

        // Draw active stroke in progress
        if let Some(stroke) = &self.active_stroke {
            self.draw_strokes(&ctx2d, std::slice::from_ref(stroke));
        }

        // Crop overlay on top
        if self.mode == EditorMode::Crop
            && let (Some(start), Some(end)) = (self.crop_start, self.crop_end)
        {
            self.draw_crop_overlay(&ctx2d, &canvas, start, end);
        }
    }

    fn draw_strokes(&self, ctx2d: &web_sys::CanvasRenderingContext2d, strokes: &[Vec<(f64, f64)>]) {
        ctx2d.set_stroke_style_str("#FF0000");
        ctx2d.set_line_width(3.0);
        ctx2d.set_line_cap("round");
        ctx2d.set_line_join("round");

        for stroke in strokes {
            if stroke.len() < 2 {
                continue;
            }
            ctx2d.begin_path();
            ctx2d.move_to(stroke[0].0, stroke[0].1);
            for &(x, y) in &stroke[1..] {
                ctx2d.line_to(x, y);
            }
            ctx2d.stroke();
        }
    }

    fn draw_crop_overlay(
        &self,
        ctx2d: &web_sys::CanvasRenderingContext2d,
        canvas: &HtmlCanvasElement,
        start: (f64, f64),
        end: (f64, f64),
    ) {
        let cw = canvas.width() as f64;
        let ch = canvas.height() as f64;

        let x = start.0.min(end.0);
        let y = start.1.min(end.1);
        let w = (end.0 - start.0).abs();
        let h = (end.1 - start.1).abs();

        // Dim the area outside the crop rectangle using four grey strips
        ctx2d.set_fill_style_str("rgba(0,0,0,0.5)");
        // Top strip
        ctx2d.fill_rect(0.0, 0.0, cw, y);
        // Bottom strip
        ctx2d.fill_rect(0.0, y + h, cw, ch - (y + h));
        // Left strip
        ctx2d.fill_rect(0.0, y, x, h);
        // Right strip
        ctx2d.fill_rect(x + w, y, cw - (x + w), h);

        // White border around selection
        ctx2d.set_stroke_style_str("#FFFFFF");
        ctx2d.set_line_width(2.0);
        ctx2d.set_line_cap("butt");
        ctx2d.begin_path();
        ctx2d.rect(x, y, w, h);
        ctx2d.stroke();

        // Corner handles
        let handle = 12.0;
        ctx2d.set_fill_style_str("#FFFFFF");
        // Top-left
        ctx2d.fill_rect(x - 2.0, y - 2.0, handle, 3.0);
        ctx2d.fill_rect(x - 2.0, y - 2.0, 3.0, handle);
        // Top-right
        ctx2d.fill_rect(x + w - handle + 2.0, y - 2.0, handle, 3.0);
        ctx2d.fill_rect(x + w - 1.0, y - 2.0, 3.0, handle);
        // Bottom-left
        ctx2d.fill_rect(x - 2.0, y + h - 1.0, handle, 3.0);
        ctx2d.fill_rect(x - 2.0, y + h - handle + 2.0, 3.0, handle);
        // Bottom-right
        ctx2d.fill_rect(x + w - handle + 2.0, y + h - 1.0, handle, 3.0);
        ctx2d.fill_rect(x + w - 1.0, y + h - handle + 2.0, 3.0, handle);
    }

    /// Bake all strokes into the base image and clear the stroke list.
    /// Returns the new data URL from the canvas.
    fn bake_strokes(&mut self) -> Option<String> {
        let canvas = self.canvas()?;
        let img = self.loaded_img.as_ref()?;
        let ctx2d = match canvas.get_context("2d") {
            Ok(Some(obj)) => obj.unchecked_into::<web_sys::CanvasRenderingContext2d>(),
            _ => return None,
        };

        let _ = ctx2d.draw_image_with_html_image_element(img, 0.0, 0.0);
        self.draw_strokes(&ctx2d, &self.strokes);
        self.strokes.clear();
        self.active_stroke = None;

        canvas
            .to_data_url_with_type_and_encoder_options(
                "image/jpeg",
                &JsValue::from_f64(JPEG_QUALITY),
            )
            .ok()
    }
}

/// Async: rotate the given data URL by ±90° and return the new data URL.
async fn rotate_url(data_url: String, clockwise: bool) -> Result<String, String> {
    let document = web_sys::window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    let img: HtmlImageElement = document
        .create_element("img")
        .map_err(|e| format!("{:?}", e))?
        .unchecked_into();

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

    img.set_src(&data_url);
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| format!("{:?}", e))?;

    let w = img.natural_width() as f64;
    let h = img.natural_height() as f64;

    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .map_err(|e| format!("{:?}", e))?
        .unchecked_into();
    canvas.set_width(h as u32);
    canvas.set_height(w as u32);

    let ctx2d: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("no context")?
        .unchecked_into();

    ctx2d
        .translate(h / 2.0, w / 2.0)
        .map_err(|e| format!("{:?}", e))?;
    let angle = if clockwise {
        std::f64::consts::FRAC_PI_2
    } else {
        -std::f64::consts::FRAC_PI_2
    };
    ctx2d.rotate(angle).map_err(|e| format!("{:?}", e))?;
    ctx2d
        .draw_image_with_html_image_element(&img, -w / 2.0, -h / 2.0)
        .map_err(|e| format!("{:?}", e))?;

    canvas
        .to_data_url_with_type_and_encoder_options("image/jpeg", &JsValue::from_f64(JPEG_QUALITY))
        .map_err(|e| format!("{:?}", e))
}

/// Async: crop a region from the given data URL.
async fn crop_url(data_url: String, x: f64, y: f64, w: f64, h: f64) -> Result<String, String> {
    let document = web_sys::window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    let img: HtmlImageElement = document
        .create_element("img")
        .map_err(|e| format!("{:?}", e))?
        .unchecked_into();

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

    img.set_src(&data_url);
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| format!("{:?}", e))?;

    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .map_err(|e| format!("{:?}", e))?
        .unchecked_into();
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);

    let ctx2d: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("no context")?
        .unchecked_into();

    ctx2d
        .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &img, x, y, w, h, 0.0, 0.0, w, h,
        )
        .map_err(|e| format!("{:?}", e))?;

    canvas
        .to_data_url_with_type_and_encoder_options("image/jpeg", &JsValue::from_f64(JPEG_QUALITY))
        .map_err(|e| format!("{:?}", e))
}

impl Component for PhotoEditor {
    type Message = Msg;
    type Properties = PhotoEditorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let src = ctx.props().src.clone();
        let link = ctx.link().clone();

        // Kick off initial image load
        let editor = Self {
            canvas_ref: NodeRef::default(),
            base_data_url: src.clone(),
            loaded_img: None,
            strokes: Vec::new(),
            active_stroke: None,
            mode: EditorMode::Idle,
            crop_start: None,
            crop_end: None,
        };

        // Load image immediately via a message
        link.send_message(Msg::SetBase(src));

        editor
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetBase(url) => {
                self.base_data_url = url.clone();
                self.crop_start = None;
                self.crop_end = None;

                let document = match web_sys::window().and_then(|w| w.document()) {
                    Some(d) => d,
                    None => return false,
                };

                let img: HtmlImageElement = match document
                    .create_element("img")
                    .ok()
                    .and_then(|el| el.dyn_into::<HtmlImageElement>().ok())
                {
                    Some(i) => i,
                    None => return false,
                };

                let link = ctx.link().clone();
                let img_clone = img.clone();
                let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                    let resolve_inner = resolve.clone();
                    let link_inner = link.clone();
                    let onload = Closure::once(Box::new(move || {
                        link_inner.send_message(Msg::Loaded);
                        let _ = resolve_inner.call1(&JsValue::NULL, &JsValue::NULL);
                    }) as Box<dyn FnOnce()>);
                    img_clone.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                });
                // Keep promise alive but we don't need the result
                let _ = promise;

                img.set_src(&url);
                self.loaded_img = Some(img);
                false
            }

            Msg::Loaded => {
                let canvas = match self.canvas() {
                    Some(c) => c,
                    None => return false,
                };
                let img = match &self.loaded_img {
                    Some(i) => i,
                    None => return false,
                };

                canvas.set_width(img.natural_width());
                canvas.set_height(img.natural_height());
                self.redraw_canvas();
                true
            }

            Msg::InputDown(x, y) => {
                match self.mode {
                    EditorMode::Draw => {
                        self.active_stroke = Some(vec![(x, y)]);
                        // Begin drawing on canvas
                        if let Some(canvas) = self.canvas()
                            && let Ok(Some(obj)) = canvas.get_context("2d")
                        {
                            let ctx2d: web_sys::CanvasRenderingContext2d = obj.unchecked_into();
                            ctx2d.set_stroke_style_str("#FF0000");
                            ctx2d.set_line_width(3.0);
                            ctx2d.set_line_cap("round");
                            ctx2d.set_line_join("round");
                            ctx2d.begin_path();
                            ctx2d.move_to(x, y);
                        }
                    }
                    EditorMode::Crop => {
                        self.crop_start = Some((x, y));
                        self.crop_end = None;
                    }
                    EditorMode::Idle => {}
                }
                false
            }

            Msg::InputMove(x, y) => {
                match self.mode {
                    EditorMode::Draw => {
                        if let Some(stroke) = &mut self.active_stroke {
                            stroke.push((x, y));
                            // Draw incrementally on canvas
                            if let Some(canvas) = self.canvas()
                                && let Ok(Some(obj)) = canvas.get_context("2d")
                            {
                                let ctx2d: web_sys::CanvasRenderingContext2d = obj.unchecked_into();
                                ctx2d.line_to(x, y);
                                ctx2d.stroke();
                                ctx2d.begin_path();
                                ctx2d.move_to(x, y);
                            }
                        }
                    }
                    EditorMode::Crop => {
                        if self.crop_start.is_some() {
                            self.crop_end = Some((x, y));
                            self.redraw_canvas();
                        }
                    }
                    EditorMode::Idle => {}
                }
                false
            }

            Msg::InputUp => {
                match self.mode {
                    EditorMode::Draw => {
                        if let Some(stroke) = self.active_stroke.take()
                            && stroke.len() >= 2
                        {
                            self.strokes.push(stroke);
                        }
                        false
                    }
                    // Re-render so the "Apply Crop" button appears in the HTML
                    EditorMode::Crop => self.crop_end.is_some(),
                    EditorMode::Idle => false,
                }
            }

            Msg::SetMode(mode) => {
                self.mode = mode;
                self.crop_start = None;
                self.crop_end = None;
                self.redraw_canvas();
                true
            }

            Msg::RotateCw => {
                let baked = self
                    .bake_strokes()
                    .unwrap_or_else(|| self.base_data_url.clone());
                let link = ctx.link().clone();
                spawn_local(async move {
                    match rotate_url(baked, true).await {
                        Ok(url) => link.send_message(Msg::SetBase(url)),
                        Err(_) => link.send_message(Msg::Noop),
                    }
                });
                false
            }

            Msg::RotateCcw => {
                let baked = self
                    .bake_strokes()
                    .unwrap_or_else(|| self.base_data_url.clone());
                let link = ctx.link().clone();
                spawn_local(async move {
                    match rotate_url(baked, false).await {
                        Ok(url) => link.send_message(Msg::SetBase(url)),
                        Err(_) => link.send_message(Msg::Noop),
                    }
                });
                false
            }

            Msg::ApplyCrop => {
                let (start, end) = match (self.crop_start, self.crop_end) {
                    (Some(s), Some(e)) => (s, e),
                    _ => return false,
                };

                let x = start.0.min(end.0);
                let y = start.1.min(end.1);
                let w = (end.0 - start.0).abs();
                let h = (end.1 - start.1).abs();

                if w < 4.0 || h < 4.0 {
                    return false;
                }

                let baked = self
                    .bake_strokes()
                    .unwrap_or_else(|| self.base_data_url.clone());
                let link = ctx.link().clone();
                spawn_local(async move {
                    match crop_url(baked, x, y, w, h).await {
                        Ok(url) => link.send_message(Msg::SetBase(url)),
                        Err(_) => link.send_message(Msg::Noop),
                    }
                });
                false
            }

            Msg::UsePhoto => {
                // Bake any remaining strokes then export
                let _ = self.bake_strokes();
                if let Some(canvas) = self.canvas() {
                    let data_url = canvas
                        .to_data_url_with_type_and_encoder_options(
                            "image/jpeg",
                            &JsValue::from_f64(JPEG_QUALITY),
                        )
                        .unwrap_or_default();
                    ctx.props().on_done.emit(data_url);
                }
                false
            }

            Msg::Cancel => {
                ctx.props().on_cancel.emit(());
                false
            }

            Msg::Noop => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        // Toolbar button callbacks
        let on_rotate_ccw = link.callback(|_: MouseEvent| Msg::RotateCcw);
        let on_rotate_cw = link.callback(|_: MouseEvent| Msg::RotateCw);
        let on_mode_idle = link.callback(|_: MouseEvent| Msg::SetMode(EditorMode::Idle));
        let on_mode_draw = link.callback(|_: MouseEvent| Msg::SetMode(EditorMode::Draw));
        let on_mode_crop = link.callback(|_: MouseEvent| Msg::SetMode(EditorMode::Crop));
        let on_apply_crop = link.callback(|_: MouseEvent| Msg::ApplyCrop);
        let on_cancel_crop = link.callback(|_: MouseEvent| Msg::SetMode(EditorMode::Idle));
        let on_use = link.callback(|_: MouseEvent| Msg::UsePhoto);
        let on_cancel = link.callback(|_: MouseEvent| Msg::Cancel);

        // Draw mode active class
        let draw_class = if self.mode == EditorMode::Draw {
            "btn-icon active"
        } else {
            "btn-icon"
        };
        let crop_class = if self.mode == EditorMode::Crop {
            "btn-icon active"
        } else {
            "btn-icon"
        };

        // Canvas pointer events
        let canvas_ref = self.canvas_ref.clone();
        let on_ptr_down = {
            let canvas_ref = canvas_ref.clone();
            link.callback(move |e: PointerEvent| {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    let (x, y) = PhotoEditor::client_to_canvas(
                        &canvas,
                        e.client_x() as f64,
                        e.client_y() as f64,
                    );
                    Msg::InputDown(x, y)
                } else {
                    Msg::Noop
                }
            })
        };
        let on_ptr_move = {
            let canvas_ref = canvas_ref.clone();
            link.callback(move |e: PointerEvent| {
                if e.buttons() == 0 {
                    return Msg::Noop;
                }
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    let (x, y) = PhotoEditor::client_to_canvas(
                        &canvas,
                        e.client_x() as f64,
                        e.client_y() as f64,
                    );
                    Msg::InputMove(x, y)
                } else {
                    Msg::Noop
                }
            })
        };
        let on_ptr_up = link.callback(|_: PointerEvent| Msg::InputUp);

        // Touch events (supplementary for iOS Safari edge cases)
        let on_touch_start = {
            let canvas_ref = canvas_ref.clone();
            link.callback(move |e: TouchEvent| {
                e.prevent_default();
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>()
                    && let Some(touch) = e.touches().get(0)
                {
                    let (x, y) = PhotoEditor::client_to_canvas(
                        &canvas,
                        touch.client_x() as f64,
                        touch.client_y() as f64,
                    );
                    return Msg::InputDown(x, y);
                }
                Msg::Noop
            })
        };
        let on_touch_move = {
            let canvas_ref = canvas_ref.clone();
            link.callback(move |e: TouchEvent| {
                e.prevent_default();
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>()
                    && let Some(touch) = e.touches().get(0)
                {
                    let (x, y) = PhotoEditor::client_to_canvas(
                        &canvas,
                        touch.client_x() as f64,
                        touch.client_y() as f64,
                    );
                    return Msg::InputMove(x, y);
                }
                Msg::Noop
            })
        };
        let on_touch_end = link.callback(|e: TouchEvent| {
            e.prevent_default();
            Msg::InputUp
        });

        let has_crop = self.crop_end.is_some();

        html! {
            <div class="photo-editor">
                // Toolbar
                <div class="editor-toolbar">
                    <button type="button" class="btn-icon" onclick={on_rotate_ccw} title="Rotate left">
                        <span class="material-icons">{"rotate_left"}</span>
                    </button>
                    <button type="button" class="btn-icon" onclick={on_rotate_cw} title="Rotate right">
                        <span class="material-icons">{"rotate_right"}</span>
                    </button>
                    <div class="divider" />
                    <button type="button" class={draw_class} onclick={on_mode_draw} title="Draw">
                        <span class="material-icons">{"edit"}</span>
                    </button>
                    <button type="button" class={crop_class} onclick={on_mode_crop} title="Crop">
                        <span class="material-icons">{"crop"}</span>
                    </button>
                    if self.mode == EditorMode::Draw || self.mode == EditorMode::Crop {
                        <button type="button" class="btn-icon" onclick={on_mode_idle} title="Cancel tool">
                            <span class="material-icons">{"close"}</span>
                        </button>
                    }
                </div>

                // Canvas
                <div class="editor-canvas-wrapper">
                    <canvas
                        ref={self.canvas_ref.clone()}
                        class="editor-canvas"
                        onpointerdown={on_ptr_down}
                        onpointermove={on_ptr_move}
                        onpointerup={on_ptr_up}
                        ontouchstart={on_touch_start}
                        ontouchmove={on_touch_move}
                        ontouchend={on_touch_end}
                    />
                </div>

                // Crop apply bar
                if self.mode == EditorMode::Crop && has_crop {
                    <div class="editor-actions">
                        <div class="editor-crop-actions">
                            <button type="button" class="btn btn-primary" onclick={on_apply_crop}>
                                <span class="material-icons">{"check"}</span>
                                {" Apply"}
                            </button>
                            <button type="button" class="btn btn-secondary" onclick={on_cancel_crop}>
                                <span class="material-icons">{"close"}</span>
                                {" Cancel"}
                            </button>
                        </div>
                    </div>
                }

                // Bottom actions
                <div class="editor-actions">
                    <button type="button" class="btn btn-secondary" onclick={on_cancel}>{"Cancel"}</button>
                    <button type="button" class="btn btn-primary" onclick={on_use}>
                        <span class="material-icons">{"check"}</span>
                        {" Use Photo"}
                    </button>
                </div>
            </div>
        }
    }
}
