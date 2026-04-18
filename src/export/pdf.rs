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

/// Returns a JavaScript string literal for `s`, using JSON encoding.
///
/// JSON string encoding is a strict subset of JavaScript string literals and
/// correctly handles all Unicode code points, including characters that
/// Rust's `{:?}` debug format emits as `\u{XXXX}` (which is Rust-only syntax
/// and would cause a JS syntax error inside an `eval`'d script).
fn js_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

fn build_jspdf_script(
    trip: &Trip,
    receipts: &[Receipt],
    categories: &[Category],
    photos: &std::collections::HashMap<String, String>,
) -> String {
    use std::collections::BTreeMap;

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

    // Per-currency totals (BTreeMap = alphabetically stable)
    let mut currency_totals: BTreeMap<String, f64> = BTreeMap::new();
    for r in receipts {
        *currency_totals.entry(r.currency.clone()).or_default() += r.amount;
    }
    let is_multi_currency = currency_totals.len() > 1;

    let currencies_label = currency_totals
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(" / ");
    let totals_summary = currency_totals
        .iter()
        .map(|(c, a)| format!("{} {:.2}", c, a))
        .collect::<Vec<_>>()
        .join("  |  ");

    // Group receipts by date → (label_key → (currency, amount)).
    // When multi-currency the label key is "Cat (EUR)" so amounts in
    // different currencies never get summed together.
    let mut daily: BTreeMap<chrono::NaiveDate, BTreeMap<String, (String, f64)>> = BTreeMap::new();
    for r in receipts {
        let cat_name = categories
            .iter()
            .find(|c| c.id == r.category_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Other".to_string());
        let label = if is_multi_currency {
            format!("{} ({})", cat_name, r.currency)
        } else {
            cat_name
        };
        let entry = daily
            .entry(r.date)
            .or_default()
            .entry(label)
            .or_insert_with(|| (r.currency.clone(), 0.0));
        entry.1 += r.amount;
    }

    // Overall per-(category, currency) totals, sorted descending by amount
    let mut cat_totals_map: BTreeMap<(String, String), f64> = BTreeMap::new();
    for r in receipts {
        let cat_name = categories
            .iter()
            .find(|c| c.id == r.category_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Other".to_string());
        *cat_totals_map
            .entry((cat_name, r.currency.clone()))
            .or_default() += r.amount;
    }
    let mut cat_totals: Vec<(String, String, f64)> = cat_totals_map
        .into_iter()
        .map(|((name, cur), amt)| (name, cur, amt))
        .collect();
    cat_totals.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

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
        "  doc.setFontSize(18); doc.setFont(undefined, 'bold'); doc.text({}, 14, 20);",
        js_str(&trip.name)
    ));
    lines.push(format!(
        "  doc.setFontSize(11); doc.setFont(undefined, 'normal'); doc.text({}, 14, 28);",
        js_str(&format!(
            "{} — {} to {}",
            currencies_label, trip.start_date, trip.end_date
        ))
    ));
    lines.push(format!(
        "  doc.setFontSize(10); doc.text({}, 14, 34);",
        js_str(&format!(
            "{} day trip  •  {}",
            trip_days as i64, totals_summary
        ))
    ));
    lines.push("  var y = 44;".to_string());

    // ── Daily breakdown ──────────────────────────────────────────────────────
    lines.push("  doc.setFont(undefined, 'bold'); doc.setFontSize(13);".to_string());
    lines.push("  doc.text('Daily Breakdown', 14, y); y += 8;".to_string());
    lines.push("  doc.setFontSize(10);".to_string());

    for (date, cats_map) in &daily {
        let display_date = date.format("%a %d %b %Y").to_string();
        let mut day_rows: Vec<(&str, &str, f64)> = cats_map
            .iter()
            .map(|(k, (cur, v))| (k.as_str(), cur.as_str(), *v))
            .collect();
        day_rows.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Per-currency day totals
        let mut day_cur_totals: BTreeMap<String, f64> = BTreeMap::new();
        for (_, cur, amt) in &day_rows {
            *day_cur_totals.entry((*cur).to_string()).or_default() += amt;
        }
        let day_total_str = day_cur_totals
            .iter()
            .map(|(c, a)| format!("{} {:.2}", c, a))
            .collect::<Vec<_>>()
            .join(" + ");

        // Estimate the height this day block needs and page-break if necessary
        let block_h = 7 + day_rows.len() as i32 * 6 + 8;
        lines.push(format!(
            "  if (y + {} > 277) {{ doc.addPage(); y = 20; }}",
            block_h
        ));

        // Date heading
        lines.push("  doc.setFont(undefined, 'bold');".to_string());
        lines.push(format!(
            "  doc.text({}, 14, y); y += 7;",
            js_str(&display_date)
        ));

        // One row per (category, currency)
        lines.push("  doc.setFont(undefined, 'normal');".to_string());
        for (label, cur, amount) in &day_rows {
            let amount_str = format!("{} {:.2}", cur, amount);
            lines.push(format!(
                "  doc.text({}, 22, y); doc.text({}, 130, y); y += 6;",
                js_str(&truncate(label, 30)),
                js_str(&amount_str)
            ));
        }

        // Day total
        lines.push("  doc.setFont(undefined, 'bold');".to_string());
        lines.push(format!(
            "  doc.text('Day total', 22, y); doc.text({}, 130, y); y += 10;",
            js_str(&day_total_str)
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
        if is_multi_currency {
            lines.push("  doc.text('Currency', 105, y);".to_string());
            lines.push("  doc.text('Total', 130, y);".to_string());
            lines.push("  doc.text('Avg / day', 162, y);".to_string());
        } else {
            let cur = currency_totals
                .keys()
                .next()
                .map(String::as_str)
                .unwrap_or("");
            lines.push(format!(
                "  doc.text({}, 115, y);",
                js_str(&format!("Total ({})", cur))
            ));
            lines.push(format!(
                "  doc.text({}, 157, y);",
                js_str(&format!("Avg / day ({})", cur))
            ));
        }
        lines.push("  doc.setFont(undefined, 'normal');".to_string());
        lines.push("  y += 4; doc.line(14, y, 196, y); y += 6;".to_string());

        for (name, cur, cat_total) in &cat_totals {
            let per_day = cat_total / trip_days;
            lines.push("  if (y > 270) { doc.addPage(); y = 20; }".to_string());
            if is_multi_currency {
                lines.push(format!(
                    "  doc.text({}, 14, y); doc.text({}, 105, y); doc.text({}, 130, y); doc.text({}, 162, y); y += 7;",
                    js_str(&truncate(name, 28)),
                    js_str(cur),
                    js_str(&format!("{:.2}", cat_total)),
                    js_str(&format!("{:.2}", per_day))
                ));
            } else {
                lines.push(format!(
                    "  doc.text({}, 14, y); doc.text({}, 115, y); doc.text({}, 157, y); y += 7;",
                    js_str(&truncate(name, 32)),
                    js_str(&format!("{:.2}", cat_total)),
                    js_str(&format!("{:.2}", per_day))
                ));
            }
        }

        // Grand total row(s)
        lines.push("  y += 2; doc.line(14, y, 196, y); y += 6;".to_string());
        lines.push("  doc.setFont(undefined, 'bold');".to_string());
        if is_multi_currency {
            lines.push(format!(
                "  doc.text('Totals', 14, y); doc.text({}, 105, y); y += 7;",
                js_str(&totals_summary)
            ));
        } else {
            let grand_total: f64 = receipts.iter().map(|r| r.amount).sum();
            lines.push(format!(
                "  doc.text('Grand Total', 14, y); doc.text({}, 115, y); doc.text({}, 157, y);",
                js_str(&format!("{:.2}", grand_total)),
                js_str(&format!("{:.2}", grand_total / trip_days))
            ));
        }
        lines.push("  doc.setFont(undefined, 'normal');".to_string());
    }

    // ── PAGE 2+: RECEIPT DETAIL ──────────────────────────────────────────────
    lines.push("  doc.addPage(); y = 20;".to_string());

    // Page heading
    lines.push(format!(
        "  doc.setFont(undefined, 'bold'); doc.setFontSize(14); doc.text({}, 14, y); y += 10;",
        js_str(&format!("{} — Receipt Detail", trip.name))
    ));

    // Table header
    lines.push("  doc.setFontSize(10);".to_string());
    lines.push("  doc.text('Date', 14, y);".to_string());
    lines.push("  doc.text('Category', 40, y);".to_string());
    lines.push("  doc.text('Amount', 100, y);".to_string());
    lines.push("  doc.text('Notes', 130, y);".to_string());
    lines.push("  doc.setFont(undefined, 'normal');".to_string());
    lines.push("  y += 4; doc.line(14, y, 196, y); y += 4;".to_string());

    for r in receipts {
        let cat = categories
            .iter()
            .find(|c| c.id == r.category_id)
            .map(|c| c.name.as_str())
            .unwrap_or("Other");
        let notes = r.notes.as_deref().unwrap_or("");
        let amount_str = format!("{} {:.2}", r.currency, r.amount);

        lines.push("  if (y > 260) { doc.addPage(); y = 20; }".to_string());
        lines.push(format!(
            "  doc.text({}, 14, y);",
            js_str(&r.date.to_string())
        ));
        lines.push(format!(
            "  doc.text({}, 40, y);",
            js_str(&truncate(cat, 25))
        ));
        lines.push(format!("  doc.text({}, 100, y);", js_str(&amount_str)));
        lines.push(format!(
            "  doc.text({}, 130, y);",
            js_str(&truncate(notes, 35))
        ));
        lines.push("  y += 7;".to_string());

        // Embed photo if available, preserving original aspect ratio
        if let Some(photo_data) = photos.get(&r.id) {
            lines.push(format!(
                "  try {{\
                    var _imgData = {};\
                    var _props = doc.getImageProperties(_imgData);\
                    var _maxW = 182; var _maxH = 200;\
                    var _ratio = Math.min(_maxW / _props.width, _maxH / _props.height);\
                    var _imgW = _props.width * _ratio;\
                    var _imgH = _props.height * _ratio;\
                    if (y + _imgH > 277) {{ doc.addPage(); y = 20; }}\
                    doc.addImage(_imgData, 'JPEG', 14, y, _imgW, _imgH);\
                    y += _imgH + 5;\
                  }} catch(e) {{}}",
                js_str(photo_data)
            ));
        }
    }

    // Total row(s)
    lines.push("  if (y > 260) { doc.addPage(); y = 20; }".to_string());
    lines.push("  y += 2; doc.line(14, y, 196, y); y += 6;".to_string());
    lines.push("  doc.setFont(undefined, 'bold');".to_string());
    lines.push(format!(
        "  doc.text({}, 100, y);",
        js_str(&format!("Total: {}", totals_summary))
    ));
    lines.push("  doc.setFont(undefined, 'normal');".to_string());

    lines.push(format!("  doc.save({});", js_str(&filename)));
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
