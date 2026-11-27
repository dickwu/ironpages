//! The PDF standard (Base-14) fonts a document actually selects.
//!
//! Every content stream shares one font resource dictionary. Listing only the
//! standard fonts some stream selects with `Tf` keeps unused fonts out of the
//! document (and out of `pdffonts` reports).

use std::collections::HashMap;
use std::io::Read as _;

/// Standard fonts the renderer can select, in resource-dictionary order.
const STANDARD_FONTS: [&str; 13] = [
    // Helvetica (sans-serif)
    "Helvetica",
    "Helvetica-Bold",
    "Helvetica-Oblique",
    "Helvetica-BoldOblique",
    // Times Roman (serif)
    "Times-Roman",
    "Times-Bold",
    "Times-Italic",
    "Times-BoldItalic",
    // Courier (monospace)
    "Courier",
    "Courier-Bold",
    "Courier-Oblique",
    "Courier-BoldOblique",
    // Symbol (math/Greek)
    "Symbol",
];

/// Which [`STANDARD_FONTS`] the document's content streams select.
#[derive(Debug, Default)]
pub(super) struct StandardFontUsage {
    used: [bool; STANDARD_FONTS.len()],
}

impl StandardFontUsage {
    /// Scan every content-bearing object: page content streams, form
    /// XObjects, patterns, and Type 3 glyph procedures. Image and embedded
    /// font-program streams cannot select fonts and are skipped.
    pub(super) fn of_objects(objects: &[String], binary_objects: &HashMap<usize, Vec<u8>>) -> Self {
        let mut usage = Self::default();
        for (index, object) in objects.iter().enumerate() {
            match binary_objects.get(&(index + 1)) {
                Some(stream) => usage.scan_stream(object, stream),
                None => usage.scan(object.as_bytes()),
            }
        }
        usage
    }

    fn scan_stream(&mut self, header: &str, stream: &[u8]) {
        if header.contains("/Subtype /Image") || header.contains("/Length1") {
            return;
        }
        if !header.contains("/FlateDecode") {
            self.scan(stream);
            return;
        }
        let mut inflated = Vec::new();
        match flate2::read::ZlibDecoder::new(stream).read_to_end(&mut inflated) {
            Ok(_) => self.scan(&inflated),
            // An undecodable stream cannot be proven font-free; keep every font.
            Err(_) => self.used = [true; STANDARD_FONTS.len()],
        }
    }

    /// Mark the fonts selected by `/<name> <size> Tf` operators.
    fn scan(&mut self, content: &[u8]) {
        let tokens = operand_tokens(content);
        for window in tokens.windows(3) {
            if window[2] != b"Tf" {
                continue;
            }
            let Some(name) = window[0].strip_prefix(b"/") else {
                continue;
            };
            if let Some(index) = STANDARD_FONTS
                .iter()
                .position(|font| font.as_bytes() == name)
            {
                self.used[index] = true;
            }
        }
    }

    /// Names of the selected standard fonts, in resource-dictionary order.
    pub(super) fn used_names(&self) -> Vec<&'static str> {
        STANDARD_FONTS
            .iter()
            .zip(self.used)
            .filter_map(|(name, used)| used.then_some(*name))
            .collect()
    }
}

/// Split PDF syntax into tokens, leaving out literal strings, hex strings and
/// comments so text that merely reads `/Courier 12 Tf` selects nothing. A
/// skipped string still separates the tokens around it.
fn operand_tokens(content: &[u8]) -> Vec<&[u8]> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut i = 0;
    while i < content.len() {
        let byte = content[i];
        let skip_to = match byte {
            b'(' => Some(literal_string_end(content, i)),
            b'<' if content.get(i + 1) != Some(&b'<') => Some(
                content[i..]
                    .iter()
                    .position(|&b| b == b'>')
                    .map_or(content.len(), |offset| i + offset + 1),
            ),
            b'%' => Some(
                content[i..]
                    .iter()
                    .position(|&b| b == b'\n' || b == b'\r')
                    .map_or(content.len(), |offset| i + offset),
            ),
            _ => None,
        };
        let boundary = skip_to.is_some() || byte.is_ascii_whitespace();
        if boundary || (byte == b'/' && start.is_some()) {
            if let Some(token_start) = start.take() {
                tokens.push(&content[token_start..i]);
            }
        }
        if let Some(end) = skip_to {
            i = end.max(i + 1);
            continue;
        }
        if !boundary && start.is_none() {
            start = Some(i);
        }
        i += 1;
    }
    if let Some(token_start) = start {
        tokens.push(&content[token_start..]);
    }
    tokens
}

/// Index just past the literal string opening at `open`, honoring nested
/// parentheses and backslash escapes.
fn literal_string_end(content: &[u8], open: usize) -> usize {
    let mut depth = 0usize;
    let mut i = open;
    while i < content.len() {
        match content[i] {
            b'\\' => i += 1,
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    content.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_operators_inside_strings_and_comments_select_nothing() {
        let usage = StandardFontUsage::of_objects(
            &[
                concat!(
                    "1 0 obj\n<< /Length 90 >>\nstream\nBT\n/Times-Roman 12 Tf\n",
                    "(x /Courier 12 Tf \\) (y\\)) Tj <2F48656C> Tj\n",
                    "% /Helvetica 12 Tf\nET\nendstream\nendobj"
                )
                .to_owned(),
                "2 0 obj\n<< /Type /Annot /Contents (a /Symbol 9 Tf b) >>\nendobj".to_owned(),
            ],
            &HashMap::new(),
        );
        assert_eq!(usage.used_names(), vec!["Times-Roman"]);
    }

    #[test]
    fn adjacent_names_split_into_tokens() {
        let usage = StandardFontUsage::of_objects(
            &["1 0 obj\nstream\n/Courier-Bold/Helvetica 10 Tf\nendstream".to_owned()],
            &HashMap::new(),
        );
        assert_eq!(usage.used_names(), vec!["Helvetica"]);
    }

    #[test]
    fn only_fonts_selected_by_tf_are_used() {
        let usage = StandardFontUsage::of_objects(
            &["1 0 obj\n<< /Length 40 >>\nstream\nBT\n/Times-Bold 12 Tf\n(x) Tj\nET\nendstream\nendobj"
                .to_owned()],
            &HashMap::new(),
        );
        assert_eq!(usage.used_names(), ["Times-Bold"]);
    }

    #[test]
    fn compressed_form_streams_are_inflated() {
        let content = b"BT /Courier 9 Tf (x) Tj ET /Symbol 10 Tf";
        let compressed = crate::render::pdf::flate_compress(content).unwrap();
        let usage = StandardFontUsage::of_objects(
            &[
                "1 0 obj\n<< /Type /XObject /Subtype /Form /Filter /FlateDecode >>\nstream\n"
                    .to_owned(),
            ],
            &HashMap::from([(1, compressed)]),
        );
        assert_eq!(usage.used_names(), ["Courier", "Symbol"]);
    }

    #[test]
    fn image_streams_and_custom_fonts_select_nothing() {
        let usage = StandardFontUsage::of_objects(
            &[
                "1 0 obj\n<< /Type /XObject /Subtype /Image >>\nstream\n".to_owned(),
                "2 0 obj\n<< /Length 20 >>\nstream\n/F1 12 Tf\nendstream\nendobj".to_owned(),
            ],
            &HashMap::from([(1, b"/Helvetica 1 Tf".to_vec())]),
        );
        assert!(usage.used_names().is_empty());
    }
}
