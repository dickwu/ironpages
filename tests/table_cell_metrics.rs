//! Table-cell block metrics, measured with sentinel bands.
//!
//! Regression coverage for the meal-plan pagination drift between Ironpages
//! 1.4.3 and the Chromium-parity engine: a paragraph inside a cell counted its
//! margins twice, an auto-width cell ignored a paragraph's intrinsic width, and
//! documents tuned against 1.4.3 metrics had no way to keep them.

use ironpages::HtmlConverter;

const SENTINEL: &str = r#"<div class="msr"></div>"#;

fn bands(converter: &HtmlConverter, units: &[&str]) -> Vec<f32> {
    let mut body = String::from(SENTINEL);
    for unit in units {
        body.push_str(unit);
        body.push_str(SENTINEL);
    }
    let html = format!(
        r##"<html><head><style>
            @page {{ size: 612pt 14000pt; margin: 20pt; }}
            body {{ margin: 0; font-size: 10pt; line-height: 12pt; }}
            .msr {{ height: 2.5pt; background-color: #010203; margin: 0; padding: 0; }}
            p {{ margin: 0 0 10pt 0; }}
        </style></head><body>{body}</body></html>"##
    );
    let tops = converter
        .measure_sentinel_tops(&html, 2.5, (1, 2, 3))
        .expect("single tall page");
    assert_eq!(
        tops.len(),
        units.len() + 1,
        "every sentinel found: {tops:?}"
    );
    tops.windows(2)
        .map(|pair| pair[1] - pair[0] - 2.5)
        .collect()
}

fn assert_close(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 0.05,
        "{what}: expected {expected}pt, got {actual}pt"
    );
}

const PARAGRAPH: &str = "<p>A paragraph measured inside the day-page table chain.</p>";

#[test]
fn paragraph_in_a_cell_counts_its_margins_once() {
    let shell = format!(
        r#"<table style="width:100%;border-collapse:collapse;border:0"><tr><td style="padding:0">{PARAGRAPH}</td></tr></table>"#
    );
    let measured = bands(&HtmlConverter::new(), &[PARAGRAPH, &shell]);
    // Bare paragraph: one 12pt line plus its 10pt bottom margin.
    assert_close(measured[0], 22.0, "bare paragraph");
    assert_close(measured[1], measured[0], "paragraph in a collapsed cell");
}

#[test]
fn paragraph_contributes_its_width_to_an_auto_cell() {
    let auto = format!(
        r#"<table style="border-collapse:collapse"><tr><td style="padding:0">{PARAGRAPH}</td></tr></table>"#
    );
    let measured = bands(&HtmlConverter::new(), &[PARAGRAPH, &auto]);
    // A shrink-to-fit column must be as wide as the paragraph's max-content,
    // so the paragraph stays on one line instead of one word per line.
    assert_close(measured[1], measured[0], "paragraph in an auto-width cell");
}

#[test]
fn legacy_metrics_give_collapsed_borders_no_layout_space() {
    let rows = r#"<table style="border-collapse:collapse;width:100%">
        <tr><td style="border:1pt solid #fff;padding:0">a</td></tr>
        <tr><td style="border:1pt solid #fff;padding:0">b</td></tr>
        <tr><td style="border:1pt solid #fff;padding:0">c</td></tr>
        <tr><td style="border:1pt solid #fff;padding:0">d</td></tr>
        <tr><td style="border:1pt solid #fff;padding:0">e</td></tr>
    </table>"#;
    let chromium = bands(&HtmlConverter::new(), &[rows]);
    let legacy = bands(
        &HtmlConverter::new().legacy_table_cell_metrics(true),
        &[rows],
    );
    assert_close(
        chromium[0],
        5.0 * 12.0 + 6.0,
        "Chromium: six 1pt grid lines",
    );
    assert_close(legacy[0], 5.0 * 12.0, "legacy: borders take no space");
}

#[test]
fn legacy_metrics_keep_only_the_cell_edge_margins() {
    let two = format!(
        r#"<table style="border-collapse:collapse;width:100%"><tr><td style="padding:0">{PARAGRAPH}{PARAGRAPH}</td></tr></table>"#
    );
    let spaced = r#"<table style="border-collapse:collapse;width:100%"><tr><td style="padding:0"><p style="margin:10pt 0 0 0">x</p><p style="margin:0 0 24pt 0">y</p><table><tr><td style="padding:0">z</td></tr></table><p style="margin:0">w</p></td></tr></table>"#;
    let chromium = bands(&HtmlConverter::new(), &[&two, spaced]);
    let legacy = bands(
        &HtmlConverter::new().legacy_table_cell_metrics(true),
        &[&two, spaced],
    );
    // Chromium: 10pt between the paragraphs and 10pt after the last one.
    assert_close(chromium[0], 2.0 * 12.0 + 20.0, "Chromium two paragraphs");
    // 1.4: the inner margin is dropped, the last bottom margin is kept.
    assert_close(legacy[0], 2.0 * 12.0 + 10.0, "legacy two paragraphs");
    // Chromium keeps the 24pt gap above the nested table; 1.4 kept only the
    // first top margin (values match Ironpress 1.4.3 output).
    assert_close(
        chromium[1],
        10.0 + 4.0 * 12.0 + 24.0,
        "Chromium gap before a table",
    );
    assert_close(legacy[1], 10.0 + 4.0 * 12.0, "legacy gap before a table");
}

#[test]
fn legacy_metrics_drop_the_ua_border_spacing() {
    let separate = r#"<table><tr><td style="padding:0">text</td></tr></table>"#;
    let chromium = bands(&HtmlConverter::new(), &[separate]);
    let legacy = bands(
        &HtmlConverter::new().legacy_table_cell_metrics(true),
        &[separate],
    );
    assert_close(
        chromium[0],
        12.0 + 3.0,
        "Chromium: 2px spacing above and below",
    );
    assert_close(legacy[0], 12.0, "legacy: no UA border-spacing");
    let authored =
        r#"<table style="border-spacing:3pt"><tr><td style="padding:0">text</td></tr></table>"#;
    let legacy = bands(
        &HtmlConverter::new().legacy_table_cell_metrics(true),
        &[authored],
    );
    assert_close(
        legacy[0],
        12.0 + 6.0,
        "legacy keeps authored border-spacing",
    );
}

#[test]
fn legacy_metrics_are_scoped_to_one_conversion() {
    let rows = r#"<table style="border-collapse:collapse;width:100%"><tr><td style="border:1pt solid;padding:0">a</td></tr></table>"#;
    let before = bands(&HtmlConverter::new(), &[rows]);
    let _ = bands(
        &HtmlConverter::new().legacy_table_cell_metrics(true),
        &[rows],
    );
    let after = bands(&HtmlConverter::new(), &[rows]);
    assert_eq!(before, after);
    assert_close(after[0], 14.0, "default metrics restored");
}
