//! Paged-layout regressions reported against workout-plan style documents:
//! named pages, table fragmentation, justified mixed-weight lines, inline
//! `white-space: nowrap`, and row-spanning cell alignment.
//!
//! Fixtures are synthetic. They draw with the PDF Base-14 fonts
//! (`FontFallback::RegisteredOnly`), so painted text stays readable as
//! literal strings in the uncompressed content streams.

use ironpages::{FontFallback, HtmlConverter, MissingGlyphs};

fn render(html: &str) -> String {
    let pdf = HtmlConverter::new()
        .compress(false)
        .font_fallback(FontFallback::RegisteredOnly(MissingGlyphs::Blank))
        .convert(html)
        .unwrap();
    String::from_utf8_lossy(&pdf).into_owned()
}

/// Body of `N 0 obj ... endobj`.
fn object(pdf: &str, id: usize) -> &str {
    let header = format!("\n{id} 0 obj");
    let start = pdf.find(&header).unwrap_or_else(|| panic!("object {id}")) + header.len();
    let end = start + pdf[start..].find("endobj").unwrap();
    &pdf[start..end]
}

fn references(list: &str) -> Vec<usize> {
    let tokens: Vec<&str> = list.split_whitespace().collect();
    tokens
        .windows(3)
        .filter(|window| window[1] == "0" && window[2].starts_with('R'))
        .filter_map(|window| window[0].trim_start_matches('[').parse().ok())
        .collect()
}

/// The content streams of every page, in page order.
fn page_contents(pdf: &str) -> Vec<String> {
    let catalog_start = pdf.find("/Type /Catalog").unwrap();
    let pages_id: usize = pdf[catalog_start..]
        .split("/Pages ")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let pages = object(pdf, pages_id);
    let kids = &pages[pages.find("/Kids [").unwrap() + 7..];
    let kids = &kids[..kids.find(']').unwrap()];
    references(kids)
        .into_iter()
        .map(|page_id| {
            let page = object(pdf, page_id);
            let contents = &page[page.find("/Contents").unwrap() + 9..];
            let contents = if contents.trim_start().starts_with('[') {
                &contents[..=contents.find(']').unwrap()]
            } else {
                &contents[..contents.find('R').unwrap() + 1]
            };
            references(contents)
                .into_iter()
                .map(|stream_id| object(pdf, stream_id).to_owned())
                .collect::<String>()
        })
        .collect()
}

/// `(x, y)` text-matrix origin of the first painted string containing `needle`.
fn text_origin(content: &str, needle: &str) -> Option<(f32, f32)> {
    let mut origin = None;
    for line in content.lines() {
        let parts: Vec<f32> = line
            .split_whitespace()
            .filter_map(|part| part.parse().ok())
            .collect();
        if line.ends_with(" Tm") && parts.len() == 6 {
            origin = Some((parts[4], parts[5]));
        }
        // Directly after `BT`, `Td` sets the origin from the identity matrix.
        if line.ends_with(" Td") && parts.len() == 2 {
            origin = Some((parts[0], parts[1]));
        }
        if (line.ends_with("Tj") || line.ends_with("TJ")) && line.contains(needle) {
            return origin;
        }
    }
    None
}

fn painted(content: &str, needle: &str) -> bool {
    content
        .lines()
        .any(|line| (line.ends_with("Tj") || line.ends_with("TJ")) && line.contains(needle))
}

#[test]
fn g6_named_first_page_lays_out_at_its_own_width() {
    let named = render(
        r#"<style>
        @page {size:A4; margin:0}
        @page narrow {margin:20mm 15mm}
        body {margin:0; font:12pt Helvetica}
        section {page:narrow}
        table {width:100%; border-collapse:collapse}
        </style><section><table><tr><td>Sentinel</td></tr></table></section>"#,
    );
    let control = render(
        r#"<style>
        @page {size:A4; margin:20mm 15mm}
        body {margin:0; font:12pt Helvetica}
        table {width:100%; border-collapse:collapse}
        </style><section><table><tr><td>Sentinel</td></tr></table></section>"#,
    );
    let text_ops = |pdf: &str| -> Vec<String> {
        page_contents(pdf)
            .concat()
            .lines()
            .filter(|line| line.ends_with(" Tf") || line.ends_with(" Tm") || line.ends_with(" cm"))
            .map(str::to_owned)
            .collect()
    };
    // Same text size and position as on a default page of the same width:
    // the content is not scaled down to fit the narrower named page.
    assert_eq!(text_ops(&named), text_ops(&control));
}

#[test]
fn g7_table_continuation_survives_a_following_named_page() {
    let pdf = render(
        r#"<style>
        @page {size:A4; margin:49mm 15mm 19mm}
        @page day {margin:49mm 15mm 19mm; @top-left {content:element(head)}}
        body {margin:0; font-family:Helvetica}
        section {page:day; break-before:page}
        header {position:running(head)}
        table {width:180mm; border-collapse:collapse; table-layout:fixed}
        th, td {height:295pt; border:1px solid; padding:1pt}
        th {height:15pt}
        tr {break-inside:avoid}
        </style>
        <section><header>Day 1</header>
        <table><thead><tr><th>Heading</th></tr></thead>
        <tbody><tr><td>Row A</td></tr><tr><td>Row B</td></tr><tr><td>Row C</td></tr></tbody></table>
        </section>
        <section><header>Day 2</header><p>Day 2 body</p></section>"#,
    );
    let pages = page_contents(&pdf);
    assert_eq!(pages.len(), 3);
    assert!(painted(&pages[0], "Row A") && painted(&pages[0], "Row B"));
    assert!(!painted(&pages[0], "Row C"));
    assert!(
        painted(&pages[1], "Heading"),
        "header repeats on the continuation"
    );
    assert!(painted(&pages[1], "Row C"), "the last row is not dropped");
    assert!(painted(&pages[2], "Day 2 body"));
}

#[test]
fn g7_too_tall_box_before_a_forced_break_is_fragmented() {
    let pdf = render(
        r#"<style>@page {size:A4; margin:49mm 15mm 19mm} body{margin:0;font-family:Helvetica}
        td {height:295pt; border:1px solid} tr {break-inside:avoid}</style>
        <table><tbody><tr><td>Row A</td></tr><tr><td>Row B</td></tr><tr><td>Row C</td></tr></tbody></table>
        <div style="break-before:page">Next</div>"#,
    );
    let pages = page_contents(&pdf);
    assert_eq!(pages.len(), 3);
    assert!(painted(&pages[1], "Row C"));
    assert!(painted(&pages[2], "Next"));
}

#[test]
fn g8_table_that_does_not_fit_splits_between_rows() {
    let rows: String = (1..=8)
        .map(|index| format!("<tr><td>Row {index}</td></tr>"))
        .collect();
    let pdf = render(&format!(
        r#"<style>
        @page {{size:A4; margin:49mm 15mm 22mm}}
        body {{margin:0; font:10pt Helvetica}}
        table {{width:180mm; border-collapse:collapse; table-layout:fixed; break-inside:auto}}
        th, td {{border:1px solid; padding:0}}
        th {{height:13.5pt}}
        td {{height:20pt}}
        tr {{break-inside:avoid}}
        </style>
        <main><div style="height:460pt">Before</div>
        <table><thead><tr><th>Week</th></tr><tr><th>Reps</th></tr></thead>
        <tbody>{rows}</tbody></table></main>"#
    ));
    let pages = page_contents(&pdf);
    assert_eq!(pages.len(), 2);
    assert!(painted(&pages[0], "Week") && painted(&pages[0], "Row 7"));
    assert!(!painted(&pages[0], "Row 8"));
    assert!(painted(&pages[1], "Week") && painted(&pages[1], "Reps"));
    assert!(painted(&pages[1], "Row 8"));
    assert!(!painted(&pages[1], "Row 1)"));
}

#[test]
fn g8_avoid_table_still_moves_whole() {
    let rows: String = (1..=10)
        .map(|index| format!("<tr><td>Row {index}</td></tr>"))
        .collect();
    let pdf = render(&format!(
        r#"<style>
        @page {{size:A4; margin:49mm 15mm 22mm}}
        body {{margin:0; font:10pt Helvetica}}
        table {{width:180mm; border-collapse:collapse; break-inside:avoid}}
        td {{height:20pt; padding:0}}
        </style>
        <div style="height:460pt">Before</div>
        <table><tbody>{rows}</tbody></table>"#
    ));
    let pages = page_contents(&pdf);
    assert_eq!(pages.len(), 2);
    assert!(!painted(&pages[0], "Row 1"), "the whole table moves");
    assert!(painted(&pages[1], "Row 1)") && painted(&pages[1], "Row 10"));
}

#[test]
fn g9_justified_run_starts_after_its_bold_lead_in() {
    let asset = |name: &str| {
        std::fs::read(format!("{}/assets/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    };
    // Registered regular and bold faces, so both runs are measured with their
    // own metrics and the line is justified.
    let gap = |align: &str| {
        let pdf = HtmlConverter::new()
            .compress(false)
            .font_fallback(FontFallback::RegisteredOnly(MissingGlyphs::Blank))
            .add_font("GfitSans", asset("LiberationSans-Regular.ttf"))
            .add_font_variant(
                "GfitSans",
                ironpages::FontFaceStyle::BOLD,
                asset("LiberationSans-Bold.ttf"),
            )
            .convert(&format!(
                r#"<style>@page {{size:A4; margin:5mm 15mm 19mm}}
                body {{margin:0; font:10pt/1.6 GfitSans}}
                div {{width:180mm; text-align:{align}}}</style>
                <div><strong>- Additional Activation Sets (Optional):</strong> Use more sets if lifting heavy weights to ensure gradual muscle recruitment.</div>"#
            ))
            .unwrap();
        let content = page_contents(&String::from_utf8_lossy(&pdf)).concat();
        // Origins of the first line's two painted runs: bold, then regular.
        let origins: Vec<(f32, f32)> = content
            .lines()
            .collect::<Vec<_>>()
            .windows(2)
            .filter(|pair| pair[0].ends_with(" Tm") && pair[1].ends_with("TJ"))
            .filter_map(|pair| {
                let parts: Vec<f32> = pair[0]
                    .split_whitespace()
                    .filter_map(|part| part.parse().ok())
                    .collect();
                (parts.len() == 6).then(|| (parts[4], parts[5]))
            })
            .collect();
        assert_eq!(origins[0].1, origins[1].1, "both runs on the first line");
        origins[1].0 - origins[0].0
    };
    // The bold run's four spaces are stretched on the justified line, so the
    // next run starts that much later than on a left-aligned line instead of
    // overlapping the bold text.
    let (justified, left) = (gap("justify"), gap("left"));
    // Four spaces in the bold lead-in are stretched; the advance should grow
    // by more than a few points, but stay well short of a second application.
    assert!(justified > left + 4.0, "{justified} vs {left}");
    assert!(
        justified < left + 40.0,
        "advance applied twice? {justified} vs {left}"
    );
}

#[test]
fn g10_inline_nowrap_span_does_not_break_at_its_hyphen() {
    let render_line = |span: &str| {
        render(&format!(
            r#"<style>@page {{size:A4; margin:5mm 15mm 19mm}}
            body {{margin:0; font:10pt/1.6 Helvetica}}
            div {{width:180mm; text-align:justify}}
            span {{white-space:nowrap}}</style>
            <div>Warm-up sets prepare your muscles and joints for heavier working sets, preventing injury, establishing a {span} connection, and priming your body for maximum performance without unnecessary fatigue.</div>"#
        ))
    };
    let unstyled = page_contents(&render_line("mind-muscle")).concat();
    assert!(
        !painted(&unstyled, "mind-muscle"),
        "the fixture must end its first line inside the compound"
    );
    let nowrap = page_contents(&render_line("<span>mind-muscle</span>")).concat();
    let (x, y) = text_origin(&nowrap, "mind-muscle").expect("painted as one piece");
    let (first_x, first_y) = text_origin(&nowrap, "Warm-up").unwrap();
    assert!(y != first_y, "moved to the second line");
    assert!(
        (x - first_x).abs() < 15.0,
        "near the start of the second line: {x} vs {first_x}"
    );
    let spaced = page_contents(&render_line("<span>mind muscle</span>")).concat();
    assert!(painted(&spaced, "mind muscle"));
}

#[test]
fn g11_middle_aligned_rowspan_centers_across_its_rows() {
    let pdf = render(
        r#"<style>@page {size:A4; margin:20mm 15mm}
        body {margin:0; font:10pt Helvetica}
        table {width:180mm; border-collapse:collapse; table-layout:fixed}
        th {height:22pt; border:1px solid; padding:0}
        .merged {vertical-align:middle}</style>
        <table><thead><tr><th rowspan="2" class="merged">Merged</th><th>Top</th></tr>
        <tr><th>Bottom</th></tr></thead><tbody><tr><td>Body</td><td>Body</td></tr></tbody></table>"#,
    );
    let content = page_contents(&pdf).concat();
    let merged = text_origin(&content, "Merged").unwrap().1;
    let top = text_origin(&content, "Top").unwrap().1;
    let bottom = text_origin(&content, "Bottom").unwrap().1;
    assert!(
        (merged - (top + bottom) / 2.0).abs() < 1.0,
        "merged {merged}, top {top}, bottom {bottom}"
    );
}

#[test]
fn g9_justified_inline_box_starts_after_stretched_spaces() {
    let asset = |name: &str| {
        std::fs::read(format!("{}/assets/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    };
    let convert = |html: &str| {
        let pdf = HtmlConverter::new()
            .compress(false)
            .font_fallback(FontFallback::RegisteredOnly(MissingGlyphs::Blank))
            .add_font("GfitSans", asset("LiberationSans-Regular.ttf"))
            .add_font_variant(
                "GfitSans",
                ironpages::FontFaceStyle::BOLD,
                asset("LiberationSans-Bold.ttf"),
            )
            .convert(html)
            .unwrap();
        page_contents(&String::from_utf8_lossy(&pdf)).concat()
    };
    let html = |align: &str| {
        format!(
            r#"<style>@page {{size:A4; margin:5mm 15mm 19mm}}
            body {{margin:0; font:10pt/1.6 GfitSans}}
            div {{width:180mm; text-align:{align}}}</style>
            <div><strong>- Additional Activation Sets (Optional):</strong><span style="display:inline-block">X</span> Use more sets if lifting heavy weights to ensure gradual muscle recruitment.</div>"#
        )
    };
    let first_line_text_and_box = |content: &str| -> (f32, Option<f32>, f32) {
        let mut origin = None;
        let mut flipped = Vec::new();
        let mut box_x = None;
        let mut first_y: Option<f32> = None;
        for line in content.lines() {
            let parts: Vec<f32> = line
                .split_whitespace()
                .filter_map(|part| part.parse().ok())
                .collect();
            if line.ends_with(" Tm") && parts.len() == 6 {
                origin = Some((parts[4], parts[5], line.starts_with("1 0 0 -1 ")));
            }
            if (line.ends_with("Tj") || line.ends_with("TJ"))
                && let Some((x, y, flipped_tm)) = origin
            {
                if flipped_tm {
                    if first_y.is_none_or(|first| (y - first).abs() < 0.5) {
                        first_y = Some(y);
                        flipped.push(x);
                    }
                } else if flipped.len() == 1 {
                    // Unflipped paint of the inline box after the bold run.
                    box_x = Some(x);
                }
                origin = None;
            }
        }
        assert!(flipped.len() >= 2, "bold and following run: {flipped:?}");
        (flipped[0], box_x, flipped[1])
    };
    let (just_bold, just_box, just_use) = first_line_text_and_box(&convert(&html("justify")));
    let (_, left_box, left_use) = first_line_text_and_box(&convert(&html("left")));
    assert!(
        just_use > left_use + 4.0,
        "justified Use {just_use} vs left {left_use}"
    );
    if let (Some(just_box), Some(left_box)) = (just_box, left_box) {
        // The inline box must advance with the stretched bold spaces, not stay
        // at the unstretched end (which would overlap the following glyphs).
        assert!(
            just_box > left_box + 4.0,
            "justified box {just_box} vs left {left_box}; Use at {just_use}"
        );
        assert!(
            just_box > just_bold + 4.0,
            "box {just_box} should start after bold {just_bold}"
        );
    }
}

#[test]
fn g6_named_first_page_recomputes_body_centering() {
    let named = render(
        r#"<style>
        @page {size:600pt 400pt; margin:0}
        @page named {margin:0 100pt}
        body {margin:0 auto; max-width:300pt; font:12pt Helvetica}
        section {page:named}
        </style><section>Sentinel</section>"#,
    );
    let control = render(
        r#"<style>
        @page {size:600pt 400pt; margin:0 100pt}
        body {margin:0 auto; max-width:300pt; font:12pt Helvetica}
        </style><section>Sentinel</section>"#,
    );
    let named_x = text_origin(&page_contents(&named).concat(), "Sentinel")
        .unwrap()
        .0;
    let control_x = text_origin(&page_contents(&control).concat(), "Sentinel")
        .unwrap()
        .0;
    // Printable width 400pt, max-width 300pt → 50pt gutters (not the 150pt
    // gutters computed for the default zero-margin page, which would start
    // the text 100pt further to the right).
    assert!(
        (named_x - control_x).abs() < 1.0,
        "named {named_x} vs control {control_x}"
    );
    assert!(named_x < 230.0, "kept the default-page gutter? x={named_x}");
}

#[test]
fn g8_forced_break_inside_an_overflowing_table_is_honored() {
    let pdf = render(
        r#"<style>
        @page {size:400pt 300pt; margin:0}
        body {margin:0; font:10pt Helvetica}
        table {width:100%; border-collapse:collapse; table-layout:fixed; break-inside:auto}
        td {height:40pt; padding:0; border:0}
        tr {break-inside:avoid}
        </style>
        <div style="height:210pt">Before</div>
        <table>
        <tr><td>Row 1</td></tr>
        <tr style="break-before:page"><td>Row 2</td></tr>
        <tr><td>Row 3</td></tr>
        <tr><td>Row 4</td></tr>
        </table>"#,
    );
    let pages = page_contents(&pdf);
    assert!(pages.len() >= 2, "pages={}", pages.len());
    assert!(painted(&pages[0], "Before") && painted(&pages[0], "Row 1"));
    assert!(
        !painted(&pages[0], "Row 2"),
        "forced break must start a new page"
    );
    assert!(painted(&pages[1], "Row 2"));
    let occurrences = |label: &str| pages.iter().filter(|page| painted(page, label)).count();
    for index in 1..=4 {
        assert_eq!(occurrences(&format!("Row {index}")), 1, "Row {index} once");
    }
}

#[test]
fn g8_table_target_anchor_stays_on_the_first_fragment() {
    let rows: String = (1..=8)
        .map(|index| format!("<tr><td>Row {index}</td></tr>"))
        .collect();
    let pdf = render(&format!(
        r##"<style>
        @page {{size:A4; margin:49mm 15mm 22mm}}
        body {{margin:0; font:10pt Helvetica}}
        a::after {{content:"P" target-counter(attr(href), page)}}
        table {{width:180mm; border-collapse:collapse; table-layout:fixed; break-inside:auto}}
        th, td {{border:1px solid; padding:0}}
        th {{height:13.5pt}}
        td {{height:20pt}}
        tr {{break-inside:avoid}}
        </style>
        <a href="#tbl">See</a>
        <div style="height:460pt">Before</div>
        <table id="tbl"><thead><tr><th>Week</th></tr><tr><th>Reps</th></tr></thead>
        <tbody>{rows}</tbody></table>"##
    ));
    let pages = page_contents(&pdf);
    assert!(painted(&pages[0], "Row 7") && painted(&pages[1], "Row 8"));
    let first = &pages[0];
    assert!(
        painted(first, "P1"),
        "anchor page is the first fragment: {first}"
    );
    assert!(
        !painted(first, "P2"),
        "must not use the continuation page: {first}"
    );
}

#[test]
fn g10_nowrap_span_keeps_nested_styled_children_together() {
    let render_line = |span: &str| {
        render(&format!(
            r#"<style>@page {{size:A4; margin:5mm 15mm 19mm}}
            body {{margin:0; font:10pt/1.6 Helvetica}}
            div {{width:180mm; text-align:justify}}
            span {{white-space:nowrap}}</style>
            <div>Warm-up sets prepare your muscles and joints for heavier working sets, preventing injury, establishing a {span} connection, and priming your body for maximum performance without unnecessary fatigue.</div>"#
        ))
    };
    let content = page_contents(&render_line("<span>alpha <b>beta</b> gamma</span>")).concat();
    let first_y = text_origin(&content, "Warm-up").unwrap().1;
    let alpha = text_origin(&content, "alpha").unwrap();
    let beta = text_origin(&content, "beta").unwrap();
    let gamma = text_origin(&content, "gamma").unwrap();
    assert_eq!(alpha.1, beta.1, "alpha and beta stay on one line");
    assert_eq!(beta.1, gamma.1, "beta and gamma stay on one line");
    assert_ne!(alpha.1, first_y, "the nowrap group moves together");
    assert!(alpha.0 < beta.0 && beta.0 < gamma.0);
}

#[test]
fn g10_nowrap_span_keeps_the_wrap_opportunity_before_it() {
    // The space before a nowrap span is an ordinary soft wrap opportunity:
    // only the span moves to the next line, not the word in front of it.
    let render_line = |span: &str| {
        render(&format!(
            r#"<style>@page {{size:A4; margin:5mm 15mm 19mm}}
            body {{margin:0; font:10pt/1.6 Helvetica}}
            div {{width:180mm}}
            span {{white-space:nowrap}}</style>
            <div>Warm-up sets prepare your muscles and joints for heavier working sets, preventing injury, establishing a {span} connection, and priming your body for maximum performance without unnecessary fatigue.</div>"#
        ))
    };
    let nowrap = page_contents(&render_line("<span>mind-muscle</span>")).concat();
    assert!(
        painted(&nowrap, "establishing a)"),
        "the word before the span stays on the first line"
    );
    let (_, first_y) = text_origin(&nowrap, "Warm-up").unwrap();
    let (_, span_y) = text_origin(&nowrap, "mind-muscle").unwrap();
    assert!(
        span_y != first_y,
        "the nowrap span moves to the second line"
    );
    // The same line with the span inline-block is the reference layout.
    let block = page_contents(&render_line(
        r#"<span style="display:inline-block">mind-muscle</span>"#,
    ))
    .concat();
    assert!(painted(&block, "establishing a)"));
    // The span starts the second line, at the same left edge as the first.
    let (first_x, _) = text_origin(&nowrap, "Warm-up").unwrap();
    let (span_x, _) = text_origin(&nowrap, "mind-muscle").unwrap();
    assert!((span_x - first_x).abs() < 0.01, "{span_x} vs {first_x}");
}

#[test]
fn g11_rowspan_vertical_align_includes_row_spacing() {
    let pdf = render(
        r#"<style>@page {size:A4; margin:20mm 15mm}
        body {margin:0; font:10pt Helvetica}
        table {width:180mm; border-collapse:separate; border-spacing:0 20pt; table-layout:fixed}
        th {height:22pt; padding:0; vertical-align:bottom}
        .merged {vertical-align:bottom}</style>
        <table><thead><tr><th rowspan="2" class="merged">Merged</th><th>Top</th></tr>
        <tr><th>Bottom</th></tr></thead><tbody><tr><td>Body</td><td>Body</td></tr></tbody></table>"#,
    );
    let content = page_contents(&pdf).concat();
    let merged = text_origin(&content, "Merged").unwrap().1;
    let bottom = text_origin(&content, "Bottom").unwrap().1;
    assert!(
        (merged - bottom).abs() < 1.0,
        "bottom-aligned merged {merged} vs Bottom {bottom}"
    );
}

#[test]
fn forced_break_inside_a_page_filling_table_does_not_leave_a_blank_page() {
    // A table whose first fragment fills the page and is followed by a nested
    // forced break must be placed as cut: re-fragmenting it against its own
    // estimated extent would push its content to a second page and leave the
    // first one empty.
    let pdf = render(
        r#"<style>
        @page {size:144pt 150pt; margin:0}
        body {margin:0; font:12pt Helvetica}
        .stage {break-before:page}
        .outer {display:table; border-collapse:separate; border-spacing:3px;
                width:126px; padding:7px; border:2px solid #577590;
                min-height:96px; margin:33px auto 0}
        .own {height:22px; white-space:nowrap}
        .inner {break-before:page; break-inside:avoid; width:58px; height:48px}
        </style>
        <div class="stage"><div>Lead</div></div>
        <div class="stage"><div class="outer"><div class="own">Alpha Beta</div>
        <div class="inner">Gamma</div></div></div>"#,
    );
    let pages = page_contents(&pdf);
    assert_eq!(pages.len(), 3, "pages={}", pages.len());
    assert!(painted(&pages[0], "Lead"));
    assert!(
        painted(&pages[1], "Alpha"),
        "table content stays on its page"
    );
    assert!(painted(&pages[2], "Gamma"));
}
