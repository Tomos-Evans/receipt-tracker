/// PDF export via jsPDF (loaded from CDN or bundled).
///
/// We use js_sys::eval to invoke jsPDF rather than printpdf because
/// printpdf's image encoding pulls in native codecs that don't compile
/// cleanly to wasm32 in all toolchain configurations.
///
/// The service worker caches the jsPDF script on first load so it's
/// available offline after the first network access.
use rexie::Rexie;
use crate::error::{AppError, AppResult};
use crate::models::{Trip, Receipt, Category};
use crate::storage::photos::get_photo;

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

    // Build a JS script that uses jsPDF to generate the PDF
    // jsPDF must be loaded as a <script> tag or via dynamic import
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
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
            .collect::<String>()
    );

    let mut lines = Vec::new();
    lines.push("(function() {".to_string());
    lines.push("  if (typeof jsPDF === 'undefined' && typeof window.jspdf === 'undefined') {".to_string());
    lines.push("    alert('jsPDF not loaded. Connect to the internet once to cache it.');".to_string());
    lines.push("    return;".to_string());
    lines.push("  }".to_string());
    lines.push("  var JsPDF = (typeof jsPDF !== 'undefined') ? jsPDF : window.jspdf.jsPDF;".to_string());
    lines.push("  var doc = new JsPDF({ orientation: 'portrait', unit: 'mm', format: 'a4' });".to_string());

    // Title
    lines.push(format!(
        "  doc.setFontSize(18); doc.text({:?}, 14, 20);",
        trip.name
    ));
    lines.push(format!(
        "  doc.setFontSize(11); doc.text({:?}, 14, 28);",
        format!("{} — {} to {}", trip.currency, trip.start_date, trip.end_date)
    ));

    // Table header
    lines.push("  var y = 38;".to_string());
    lines.push("  doc.setFontSize(10);".to_string());
    lines.push("  doc.setFont(undefined, 'bold');".to_string());
    lines.push("  doc.text('Date', 14, y);".to_string());
    lines.push("  doc.text('Category', 40, y);".to_string());
    lines.push("  doc.text('Amount', 100, y);".to_string());
    lines.push("  doc.text('Notes', 130, y);".to_string());
    lines.push("  doc.setFont(undefined, 'normal');".to_string());
    lines.push("  y += 4;".to_string());
    lines.push("  doc.line(14, y, 196, y);".to_string());
    lines.push("  y += 4;".to_string());

    let mut total = 0.0f64;
    for r in receipts {
        let cat = categories.iter()
            .find(|c| c.id == r.category_id)
            .map(|c| c.name.as_str())
            .unwrap_or("Other");
        let notes = r.notes.as_deref().unwrap_or("");
        let amount_str = format!("{} {:.2}", trip.currency, r.amount);
        total += r.amount;

        // New page if near bottom
        lines.push("  if (y > 260) { doc.addPage(); y = 20; }".to_string());
        lines.push(format!(
            "  doc.text({:?}, 14, y);",
            r.date.to_string()
        ));
        lines.push(format!("  doc.text({:?}, 40, y);", truncate(cat, 25)));
        lines.push(format!("  doc.text({:?}, 100, y);", amount_str));
        lines.push(format!("  doc.text({:?}, 130, y);", truncate(notes, 35)));
        lines.push("  y += 7;".to_string());

        // Embed photo if available
        if let Some(photo_data) = photos.get(&r.id) {
            lines.push("  if (y > 210) { doc.addPage(); y = 20; }".to_string());
            // Inline base64 directly — jsPDF accepts data URIs
            lines.push(format!(
                "  try {{ doc.addImage({:?}, 'JPEG', 14, y, 60, 45); y += 50; }} catch(e) {{}}",
                photo_data
            ));
        }
    }

    // Total row
    lines.push("  if (y > 260) { doc.addPage(); y = 20; }".to_string());
    lines.push("  y += 2;".to_string());
    lines.push("  doc.line(14, y, 196, y);".to_string());
    lines.push("  y += 6;".to_string());
    lines.push("  doc.setFont(undefined, 'bold');".to_string());
    lines.push(format!(
        "  doc.text('Total: {} {:.2}', 100, y);",
        trip.currency, total
    ));

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
