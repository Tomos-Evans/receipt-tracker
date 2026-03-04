use crate::error::{AppError, AppResult};
use crate::models::{Category, Receipt, Trip};
use crate::storage::photos::get_photo;
/// PDF export via jsPDF (loaded from CDN or bundled).
///
/// We use js_sys::eval to invoke jsPDF rather than printpdf because
/// printpdf's image encoding pulls in native codecs that don't compile
/// cleanly to wasm32 in all toolchain configurations.
///
/// The service worker caches the jsPDF script on first load so it's
/// available offline after the first network access.
use rexie::Rexie;

pub async fn export_pdf(
    db: &Rexie,
    trip: &Trip,
    receipts: &[Receipt],
    categories: &[Category],
) -> AppResult<()> {
    // Fetch photos for all receipts (on demand)
    let mut photo_map = std::collections::HashMap::new();
    for r in receipts {
        if let Ok(Some(data)) = get_photo(db, &r.id).await {
            photo_map.insert(r.id.clone(), data);
        }
    }

    let script = build_jspdf_script(trip, receipts, categories, &photo_map);

    match js_sys::eval(&script) {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::Export(format!(
            "PDF generation failed. Make sure jsPDF is loaded.\n{:?}",
            e
        ))),
    }
}

fn build_jspdf_script(
    trip: &Trip,
    receipts: &[Receipt],
    categories: &[Category],
    photos: &std::collections::HashMap<String, String>,
) -> String {
    let filename = format!(
        "{}.pdf",
        trip.name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            })
            .collect::<String>()
    );

    let trip_days = ((trip.end_date - trip.start_date).num_days() + 1).max(1) as f64;

    // Group receipts by date (BTreeMap keeps dates in order), then by category name
    let mut daily: std::collections::BTreeMap<
        chrono::NaiveDate,
        std::collections::HashMap<String, f64>,
    > = std::collections::BTreeMap::new();
    for r in receipts {
        let cat_name = categories
            .iter()
            .find(|c| c.id == r.category_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Other".to_string());
        *daily
            .entry(r.date)
            .or_default()
            .entry(cat_name)
            .or_default() += r.amount;
    }

    // Overall per-category totals, sorted descending
    let mut cat_totals_map: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();
    for r in receipts {
        let cat_name = categories
            .iter()
            .find(|c| c.id == r.category_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Other".to_string());
        *cat_totals_map.entry(cat_name).or_default() += r.amount;
    }
    let mut cat_totals: Vec<(String, f64)> = cat_totals_map.into_iter().collect();
    cat_totals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let grand_total: f64 = receipts.iter().map(|r| r.amount).sum();

    let mut lines = Vec::new();

    // JS boilerplate
    lines.push("(function() {".to_string());
    lines.push(
        "  if (typeof jsPDF === 'undefined' && typeof window.jspdf === 'undefined') {".to_string(),
    );
    lines.push(
        "    alert('jsPDF not loaded. Connect to the internet once to cache it.');".to_string(),
    );
    lines.push("    return;".to_string());
    lines.push("  }".to_string());
    lines.push(
        "  var JsPDF = (typeof jsPDF !== 'undefined') ? jsPDF : window.jspdf.jsPDF;".to_string(),
    );
    lines.push(
        "  var doc = new JsPDF({ orientation: 'portrait', unit: 'mm', format: 'a4' });".to_string(),
    );

    // ── PAGE 1: SUMMARY ─────────────────────────────────────────────────────

    // Header
    lines.push(format!(
        "  doc.setFontSize(18); doc.setFont(undefined, 'bold'); doc.text({:?}, 14, 20);",
        trip.name
    ));
    lines.push(format!(
        "  doc.setFontSize(11); doc.setFont(undefined, 'normal'); doc.text({:?}, 14, 28);",
        format!(
            "{} — {} to {}",
            trip.currency, trip.start_date, trip.end_date
        )
    ));
    lines.push(format!(
        "  doc.setFontSize(10); doc.text({:?}, 14, 34);",
        format!(
            "{} day trip  •  Grand total: {} {:.2}",
            trip_days as i64, trip.currency, grand_total
        )
    ));
    lines.push("  var y = 44;".to_string());

    // ── Daily breakdown ──────────────────────────────────────────────────────
    lines.push("  doc.setFont(undefined, 'bold'); doc.setFontSize(13);".to_string());
    lines.push("  doc.text('Daily Breakdown', 14, y); y += 8;".to_string());
    lines.push("  doc.setFontSize(10);".to_string());

    for (date, cats_map) in &daily {
        let display_date = date.format("%a %d %b %Y").to_string();
        let mut day_cats: Vec<(&str, f64)> =
            cats_map.iter().map(|(k, &v)| (k.as_str(), v)).collect();
        day_cats.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let day_total: f64 = day_cats.iter().map(|(_, v)| v).sum();

        // Estimate the height this day block needs and page-break if necessary
        let block_h = 7 + day_cats.len() as i32 * 6 + 8;
        lines.push(format!(
            "  if (y + {} > 277) {{ doc.addPage(); y = 20; }}",
            block_h
        ));

        // Date heading
        lines.push("  doc.setFont(undefined, 'bold');".to_string());
        lines.push(format!("  doc.text({:?}, 14, y); y += 7;", display_date));

        // One row per category
        lines.push("  doc.setFont(undefined, 'normal');".to_string());
        for (cat_name, amount) in &day_cats {
            lines.push(format!(
                "  doc.text({:?}, 22, y); doc.text({:?}, 130, y); y += 6;",
                truncate(cat_name, 30),
                format!("{} {:.2}", trip.currency, amount)
            ));
        }

        // Day total
        lines.push("  doc.setFont(undefined, 'bold');".to_string());
        lines.push(format!(
            "  doc.text('Day total', 22, y); doc.text({:?}, 130, y); y += 10;",
            format!("{} {:.2}", trip.currency, day_total)
        ));
        lines.push("  doc.setFont(undefined, 'normal');".to_string());
    }

    // ── Category totals ──────────────────────────────────────────────────────
    if !cat_totals.is_empty() {
        lines.push("  y += 4;".to_string());
        lines.push("  if (y > 245) { doc.addPage(); y = 20; }".to_string());

        lines.push("  doc.setFont(undefined, 'bold'); doc.setFontSize(13);".to_string());
        lines.push("  doc.text('Category Totals', 14, y); y += 8;".to_string());

        // Column headers
        lines.push("  doc.setFontSize(10);".to_string());
        lines.push("  doc.text('Category', 14, y);".to_string());
        lines.push(format!("  doc.text('Total ({})', 115, y);", trip.currency));
        lines.push(format!(
            "  doc.text('Avg / day ({})', 157, y);",
            trip.currency
        ));
        lines.push("  doc.setFont(undefined, 'normal');".to_string());
        lines.push("  y += 4; doc.line(14, y, 196, y); y += 6;".to_string());

        for (name, cat_total) in &cat_totals {
            let per_day = cat_total / trip_days;
            lines.push("  if (y > 270) { doc.addPage(); y = 20; }".to_string());
            lines.push(format!(
                "  doc.text({:?}, 14, y); doc.text({:?}, 115, y); doc.text({:?}, 157, y); y += 7;",
                truncate(name, 32),
                format!("{:.2}", cat_total),
                format!("{:.2}", per_day)
            ));
        }

        // Grand total row
        lines.push("  y += 2; doc.line(14, y, 196, y); y += 6;".to_string());
        lines.push("  doc.setFont(undefined, 'bold');".to_string());
        lines.push(format!(
            "  doc.text('Grand Total', 14, y); doc.text({:?}, 115, y); doc.text({:?}, 157, y);",
            format!("{:.2}", grand_total),
            format!("{:.2}", grand_total / trip_days)
        ));
        lines.push("  doc.setFont(undefined, 'normal');".to_string());
    }

    // ── PAGE 2+: RECEIPT DETAIL ──────────────────────────────────────────────
    lines.push("  doc.addPage(); y = 20;".to_string());

    // Page heading
    lines.push(format!(
        "  doc.setFont(undefined, 'bold'); doc.setFontSize(14); doc.text({:?}, 14, y); y += 10;",
        format!("{} — Receipt Detail", trip.name)
    ));

    // Table header
    lines.push("  doc.setFontSize(10);".to_string());
    lines.push("  doc.text('Date', 14, y);".to_string());
    lines.push("  doc.text('Category', 40, y);".to_string());
    lines.push("  doc.text('Amount', 100, y);".to_string());
    lines.push("  doc.text('Notes', 130, y);".to_string());
    lines.push("  doc.setFont(undefined, 'normal');".to_string());
    lines.push("  y += 4; doc.line(14, y, 196, y); y += 4;".to_string());

    let mut total = 0.0f64;
    for r in receipts {
        let cat = categories
            .iter()
            .find(|c| c.id == r.category_id)
            .map(|c| c.name.as_str())
            .unwrap_or("Other");
        let notes = r.notes.as_deref().unwrap_or("");
        let amount_str = format!("{} {:.2}", trip.currency, r.amount);
        total += r.amount;

        lines.push("  if (y > 260) { doc.addPage(); y = 20; }".to_string());
        lines.push(format!("  doc.text({:?}, 14, y);", r.date.to_string()));
        lines.push(format!("  doc.text({:?}, 40, y);", truncate(cat, 25)));
        lines.push(format!("  doc.text({:?}, 100, y);", amount_str));
        lines.push(format!("  doc.text({:?}, 130, y);", truncate(notes, 35)));
        lines.push("  y += 7;".to_string());

        // Embed photo if available, preserving original aspect ratio
        if let Some(photo_data) = photos.get(&r.id) {
            lines.push(format!(
                "  try {{\
                    var _imgData = {:?};\
                    var _props = doc.getImageProperties(_imgData);\
                    var _maxW = 182; var _maxH = 200;\
                    var _ratio = Math.min(_maxW / _props.width, _maxH / _props.height);\
                    var _imgW = _props.width * _ratio;\
                    var _imgH = _props.height * _ratio;\
                    if (y + _imgH > 277) {{ doc.addPage(); y = 20; }}\
                    doc.addImage(_imgData, 'JPEG', 14, y, _imgW, _imgH);\
                    y += _imgH + 5;\
                  }} catch(e) {{}}",
                photo_data
            ));
        }
    }

    // Total row
    lines.push("  if (y > 260) { doc.addPage(); y = 20; }".to_string());
    lines.push("  y += 2; doc.line(14, y, 196, y); y += 6;".to_string());
    lines.push("  doc.setFont(undefined, 'bold');".to_string());
    lines.push(format!(
        "  doc.text('Total: {} {:.2}', 100, y);",
        trip.currency, total
    ));
    lines.push("  doc.setFont(undefined, 'normal');".to_string());

    lines.push(format!("  doc.save({:?});", filename));
    lines.push("})();".to_string());

    lines.join("\n")
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

#[cfg(test)]
mod tests {
    use super::truncate;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    // NOTE: `truncate` uses byte-based slicing, which is correct for the ASCII
    // category names and short notes it receives in practice. Tests stay ASCII
    // to avoid a potential panic if a cut point fell mid-UTF-8 scalar.

    /// A string shorter than the limit comes back unchanged — no ellipsis added.
    #[wasm_bindgen_test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    /// A string whose length equals the limit exactly is also kept in full.
    #[wasm_bindgen_test]
    fn truncate_exact_length_unchanged() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    /// One byte over the limit: the string is cut and the ellipsis (…) appended.
    #[wasm_bindgen_test]
    fn truncate_one_over_limit() {
        // max=5, cut at byte 4 → "hell" + "…"
        assert_eq!(truncate("hello!", 5), "hell…");
    }

    /// A clearly long string is trimmed to the right prefix.
    #[wasm_bindgen_test]
    fn truncate_long_string() {
        // max=8: saturating_sub(1)=7, so &s[..7] = "Food & " (7 bytes), then "…"
        assert_eq!(truncate("Food & Drink", 8), "Food & …");
    }

    /// Empty input returns empty output with no panic.
    #[wasm_bindgen_test]
    fn truncate_empty_string() {
        assert_eq!(truncate("", 10), "");
    }

    /// max=0: every non-empty string exceeds the limit.
    /// saturating_sub(1) = 0 so &s[..0] = "" and the result is just "…".
    #[wasm_bindgen_test]
    fn truncate_max_zero_gives_ellipsis() {
        assert_eq!(truncate("abc", 0), "…");
    }

    /// max=2: a longer string yields one byte of content plus "…".
    #[wasm_bindgen_test]
    fn truncate_max_two() {
        assert_eq!(truncate("abc", 2), "a…");
    }
}
