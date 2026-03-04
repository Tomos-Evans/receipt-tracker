use crate::models::{Category, Receipt, Trip};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Blob, BlobPropertyBag, Url};

/// Build a CSV string and trigger a browser download.
pub fn export_csv(trip: &Trip, receipts: &[Receipt], categories: &[Category]) {
    let mut csv = String::new();
    csv.push_str("Date,Amount,Currency,Category,Notes\r\n");

    for r in receipts {
        let cat_name = categories
            .iter()
            .find(|c| c.id == r.category_id)
            .map(|c| c.name.as_str())
            .unwrap_or("Other");

        let notes = r.notes.as_deref().unwrap_or("").replace('"', "\"\"");
        csv.push_str(&format!(
            "{},{:.2},{},\"{}\",\"{}\"\r\n",
            r.date,
            r.amount,
            trip.currency,
            cat_name.replace('"', "\"\""),
            notes,
        ));
    }

    trigger_download(
        &csv,
        "text/csv;charset=utf-8;",
        &format!("{}.csv", sanitize_filename(&trip.name)),
    );
}

fn trigger_download(content: &str, mime: &str, filename: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    // Build Blob
    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(content));
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = match Blob::new_with_str_sequence_and_options(&array, &opts) {
        Ok(b) => b,
        Err(_) => return,
    };

    let url = match Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => return,
    };

    // Create a hidden <a> and click it
    let a = match document.create_element("a") {
        Ok(el) => el,
        Err(_) => return,
    };
    let _ = a.set_attribute("href", &url);
    let _ = a.set_attribute("download", filename);
    let a_html: web_sys::HtmlElement = a.unchecked_into();
    let _ = a_html.style().set_property("display", "none");

    if let Some(body) = document.body() {
        let _ = body.append_child(&a_html);
        a_html.click();
        let _ = body.remove_child(&a_html);
    }

    // Revoke URL after a tick
    let url_clone = url.clone();
    let closure = wasm_bindgen::closure::Closure::once(Box::new(move || {
        let _ = Url::revoke_object_url(&url_clone);
    }) as Box<dyn FnOnce()>);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        100,
    );
    closure.forget();
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    // ── sanitize_filename ─────────────────────────────────────────────────────

    /// Spaces and punctuation are replaced so the filename is safe on every OS.
    #[wasm_bindgen_test]
    fn sanitize_replaces_spaces_and_punctuation() {
        assert_eq!(sanitize_filename("Paris Trip!"), "Paris_Trip_");
    }

    /// Alphanumeric characters, hyphens, and underscores pass through unchanged.
    #[wasm_bindgen_test]
    fn sanitize_keeps_safe_chars() {
        assert_eq!(sanitize_filename("my-trip_2024"), "my-trip_2024");
    }

    /// Slashes and dots are common in user-typed names and must be sanitised.
    #[wasm_bindgen_test]
    fn sanitize_replaces_slashes_and_dots() {
        assert_eq!(sanitize_filename("Trip/2024.csv"), "Trip_2024_csv");
    }

    /// Empty input produces empty output — no panic on zero-length strings.
    #[wasm_bindgen_test]
    fn sanitize_empty_string() {
        assert_eq!(sanitize_filename(""), "");
    }

    /// Unicode letters are alphanumeric per `char::is_alphanumeric` and pass
    /// through unchanged, so non-ASCII trip names stay readable.
    #[wasm_bindgen_test]
    fn sanitize_keeps_unicode_letters() {
        assert_eq!(sanitize_filename("Zürich"), "Zürich");
    }
}
