//! Font-source policy, standard-font emission, registered font faces, and
//! viewer presentation (ironpages 2.1).
//!
//! Fixtures are synthetic and draw only with the Liberation faces bundled in
//! `assets/`, registered explicitly as caller fonts.

use ironpages::{
    FontFaceStyle, FontFallback, HtmlConverter, IronpagesError, MissingGlyphs, OpenZoom, PageLayout,
};

const MIXED_SCRIPT_HTML: &str = r#"<p style="font-family: GfitSans">Name 中文 😀 end</p>"#;

fn asset(name: &str) -> Vec<u8> {
    std::fs::read(format!("{}/assets/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn converter() -> HtmlConverter {
    HtmlConverter::new().compress(false)
}

fn text(pdf: &[u8]) -> String {
    String::from_utf8_lossy(pdf).into_owned()
}

/// Every `/BaseFont` name in the document (subset prefixes removed).
fn base_fonts(pdf: &str) -> Vec<String> {
    pdf.split("/BaseFont /")
        .skip(1)
        .map(|rest| {
            let name: String = rest
                .chars()
                .take_while(|ch| !ch.is_ascii_whitespace() && *ch != '/' && *ch != '>')
                .collect();
            name.split_once('+')
                .map_or(name.clone(), |(_, base)| base.to_owned())
        })
        .collect()
}

fn type1_fonts(pdf: &str) -> Vec<String> {
    pdf.split("/Subtype /Type1 /BaseFont /")
        .skip(1)
        .map(|rest| {
            rest.split_whitespace()
                .next()
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

fn registered_only(missing: MissingGlyphs) -> HtmlConverter {
    converter()
        .add_font("GfitSans", asset("LiberationSans-Regular.ttf"))
        .font_fallback(FontFallback::RegisteredOnly(missing))
}

#[test]
fn registered_only_embeds_only_the_registered_font() {
    let pdf = text(
        &registered_only(MissingGlyphs::Blank)
            .convert(MIXED_SCRIPT_HTML)
            .unwrap(),
    );
    let fonts = base_fonts(&pdf);
    assert!(!fonts.is_empty());
    assert!(
        fonts.iter().all(|font| font == "LiberationSans"),
        "only the registered face may be embedded: {fonts:?}"
    );
    assert_eq!(
        pdf.matches("/FontFile2").count(),
        1,
        "one embedded font program"
    );
}

#[test]
fn registered_only_leaves_missing_glyphs_blank() {
    let pdf = text(
        &registered_only(MissingGlyphs::Blank)
            .convert(MIXED_SCRIPT_HTML)
            .unwrap(),
    );
    let painted: Vec<&str> = pdf.lines().filter(|line| line.ends_with(" TJ")).collect();
    assert!(
        !painted.is_empty(),
        "the registered face still paints the Latin text"
    );
    assert!(
        painted.iter().all(|line| !line.contains("<0000>")),
        "no .notdef glyph may be painted: {painted:?}"
    );
}

/// Horizontal origins (`Tm` translation) of the painted text pieces.
fn text_origins(pdf: &str) -> Vec<f32> {
    pdf.lines()
        .filter(|line| line.ends_with(" Tm"))
        .filter_map(|line| line.split_whitespace().nth(4)?.parse().ok())
        .collect()
}

#[test]
fn registered_only_blank_keeps_the_measured_advance() {
    // A blank measures as a no-break space, so right-aligned text with a
    // blanked character starts exactly where the same line with a no-break
    // space in its place starts.
    let render = |text: &str| {
        let html =
            format!(r#"<p style="font-family: GfitSans; text-align: right; margin: 0">{text}</p>"#);
        let pdf = registered_only(MissingGlyphs::Blank)
            .convert(&html)
            .unwrap();
        text_origins(&String::from_utf8_lossy(&pdf))
    };
    let covered = render("A \u{a0} B");
    let blanked = render("A 中 B");
    assert_eq!(covered.len(), 1, "{covered:?}");
    assert_eq!(blanked.len(), 2, "'A ' and ' B' are painted: {blanked:?}");
    assert!(
        (blanked[0] - covered[0]).abs() < 0.01,
        "{blanked:?} vs {covered:?}"
    );
}

#[test]
fn registered_only_error_lists_missing_characters() {
    let error = registered_only(MissingGlyphs::Error)
        .convert(MIXED_SCRIPT_HTML)
        .unwrap_err();
    let IronpagesError::FontError(message) = &error else {
        panic!("expected a font error, got {error:?}");
    };
    for expected in ["U+4E2D '中'", "U+6587 '文'", "U+1F600 '😀'"] {
        assert!(message.contains(expected), "{message}");
    }
    assert!(!message.contains("U+0020"), "{message}");
}

#[test]
fn registered_only_error_accepts_fully_covered_text() {
    let pdf = registered_only(MissingGlyphs::Error)
        .convert(r#"<p style="font-family: GfitSans">Plain Latin text – “quoted”</p>"#)
        .unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

#[test]
fn registered_only_maps_standard_names_to_base14() {
    let pdf = text(
        &converter()
            .font_fallback(FontFallback::RegisteredOnly(MissingGlyphs::Blank))
            .convert(
                r#"<p style="font-family: Helvetica">Sans</p>
                   <p style="font-family: 'Times New Roman'"><b>Serif</b></p>
                   <p style="font-family: Courier">Mono</p>
                   <p style="font-family: sans-serif"><i>Generic</i></p>"#,
            )
            .unwrap(),
    );
    assert!(
        !pdf.contains("/FontFile"),
        "no font program may be embedded"
    );
    let mut fonts = type1_fonts(&pdf);
    fonts.sort();
    assert_eq!(
        fonts,
        ["Courier", "Helvetica", "Helvetica-Oblique", "Times-Bold"]
    );
}

#[test]
fn registered_only_blanks_unencodable_base14_characters() {
    let pdf = text(
        &converter()
            .font_fallback(FontFallback::RegisteredOnly(MissingGlyphs::Blank))
            .convert(r#"<p style="font-family: Helvetica">Name 中文 end</p>"#)
            .unwrap(),
    );
    assert!(
        !pdf.lines()
            .any(|line| line.ends_with("Tj") && line.contains('?')),
        "an unencodable character must not paint '?'"
    );
    let origins = text_origins(&pdf);
    assert_eq!(
        origins.len(),
        2,
        "'Name ' and ' end' are painted: {origins:?}"
    );

    let error = converter()
        .font_fallback(FontFallback::RegisteredOnly(MissingGlyphs::Error))
        .convert(r#"<p style="font-family: Helvetica">Name 中 end</p>"#)
        .unwrap_err();
    assert!(error.to_string().contains("U+4E2D"), "{error}");
}

#[test]
fn registered_only_base14_blank_advance_matches_layout() {
    // A blank advances exactly the distance layout measured for it, so
    // right-aligned text with blanked CJK still ends at the right edge.
    let render = |text: &str| {
        let html = format!(
            r#"<p style="font-family: Helvetica; font-size: 10pt; text-align: right; margin: 0">{text}</p>"#
        );
        let pdf = converter()
            .font_fallback(FontFallback::RegisteredOnly(MissingGlyphs::Blank))
            .convert(&html)
            .unwrap();
        text_origins(&String::from_utf8_lossy(&pdf))
    };
    let alone = render("B");
    let blanked = render("A 中文 B");
    assert_eq!(blanked.len(), 2, "{blanked:?}");
    // Helvetica's space is 278/1000 em (10pt is 13.33 user units here):
    // " B" starts one space before "B".
    let space = 0.278 * 10.0 * 4.0 / 3.0;
    assert!(
        (blanked[1] + space - alone[0]).abs() < 0.05,
        "{blanked:?} vs {alone:?}"
    );
}

#[test]
fn registered_only_applies_to_rasterized_text() {
    // Filtered text and blurred text shadows are rasterized from outlines
    // rather than written as PDF text; the missing-glyph policy still holds.
    for style in ["filter: blur(1px)", "text-shadow: 1px 1px 2px red"] {
        let html = format!(r#"<p style="font-family: GfitSans; {style}">Name 中</p>"#);
        let error = registered_only(MissingGlyphs::Error)
            .convert(&html)
            .unwrap_err();
        assert!(error.to_string().contains("U+4E2D"), "{style}: {error}");
        registered_only(MissingGlyphs::Blank)
            .convert(&html)
            .unwrap();
    }
}

#[test]
fn unused_standard_fonts_are_not_listed() {
    let pdf = text(
        &converter()
            .add_font("GfitSans", asset("LiberationSans-Regular.ttf"))
            .convert(r#"<p style="font-family: GfitSans">Only the added font</p>"#)
            .unwrap(),
    );
    assert_eq!(type1_fonts(&pdf), Vec::<String>::new());

    let pdf = text(
        &converter()
            .font_fallback(FontFallback::RegisteredOnly(MissingGlyphs::Blank))
            .footer("Page {page} of {pages}")
            .convert(r#"<p style="font-family: Courier"><b>Bold mono</b></p>"#)
            .unwrap(),
    );
    let mut fonts = type1_fonts(&pdf);
    fonts.sort();
    // The text footer paints with Helvetica; the body with Courier-Bold.
    assert_eq!(fonts, ["Courier-Bold", "Helvetica"]);
}

#[test]
fn font_variants_select_registered_bold_and_italic_faces() {
    let html =
        r#"<p style="font-family: Brand">Regular <b>Bold</b> <i>Italic</i> <b><i>Both</i></b></p>"#;
    let pdf = text(
        &converter()
            .font_fallback(FontFallback::RegisteredOnly(MissingGlyphs::Error))
            .add_font("Brand", asset("LiberationSerif-Regular.ttf"))
            .add_font_variant(
                "Brand",
                FontFaceStyle::BOLD,
                asset("LiberationSerif-Bold.ttf"),
            )
            .add_font_variant(
                "Brand",
                FontFaceStyle::ITALIC,
                asset("LiberationSerif-Italic.ttf"),
            )
            .add_font_variant(
                "Brand",
                FontFaceStyle::BOLD_ITALIC,
                asset("LiberationSerif-BoldItalic.ttf"),
            )
            .convert(html)
            .unwrap(),
    );
    let mut fonts = base_fonts(&pdf);
    fonts.sort();
    fonts.dedup();
    assert_eq!(
        fonts,
        [
            "LiberationSerif",
            "LiberationSerif-Bold",
            "LiberationSerif-BoldItalic",
            "LiberationSerif-Italic"
        ]
    );
}

#[test]
fn font_variant_matches_the_name_suffix_registration() {
    let html = r#"<p style="font-family: Brand"><b>Bold</b></p>"#;
    let variant = converter()
        .add_font("Brand", asset("LiberationSans-Regular.ttf"))
        .add_font_variant(
            "Brand",
            FontFaceStyle::from_css_weight(700, false),
            asset("LiberationSans-Bold.ttf"),
        )
        .convert(html)
        .unwrap();
    let suffix = converter()
        .add_font("Brand", asset("LiberationSans-Regular.ttf"))
        .add_font("Brand__bold", asset("LiberationSans-Bold.ttf"))
        .convert(html)
        .unwrap();
    assert_eq!(variant, suffix);
}

#[test]
fn viewer_preferences_set_page_layout_and_open_action() {
    let pdf = text(
        &converter()
            .page_layout(PageLayout::SinglePage)
            .open_zoom(OpenZoom::FitPage)
            .convert("<p>One</p><p style=\"break-before: page\">Two</p>")
            .unwrap(),
    );
    let catalog = pdf
        .split("obj\n")
        .find(|object| object.contains("/Type /Catalog"))
        .unwrap();
    assert!(catalog.contains("/PageLayout /SinglePage"), "{catalog}");
    let first_page = pdf
        .split("/Kids [")
        .nth(1)
        .and_then(|kids| kids.split(' ').next())
        .unwrap();
    assert!(
        catalog.contains(&format!("/OpenAction [{first_page} 0 R /Fit]")),
        "{catalog}"
    );

    let pdf = text(
        &converter()
            .page_layout(PageLayout::TwoPageRight)
            .open_zoom(OpenZoom::Percent(75.0))
            .convert("<p>Zoomed</p>")
            .unwrap(),
    );
    assert!(pdf.contains("/PageLayout /TwoPageRight"));
    assert!(pdf.contains("/XYZ null null 0.75]"));
}

#[test]
fn viewer_preferences_are_absent_by_default() {
    let pdf = text(&converter().convert("<p>Default</p>").unwrap());
    assert!(!pdf.contains("/PageLayout"));
    assert!(!pdf.contains("/OpenAction"));
}
