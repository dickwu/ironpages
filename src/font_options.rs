//! Font registration and font-source options for one conversion.
//!
//! [`FontFallback`] decides which fonts a conversion may draw with. The
//! default draws with registered fonts first and then falls back to host
//! system fonts, the bundled Liberation and Noto faces, and platform CJK and
//! emoji fonts. [`FontFallback::RegisteredOnly`] restricts every glyph to
//! fonts the caller registered, so the same input produces the same PDF on
//! every host. [`FontFaceStyle`] names the weight and slant of one registered
//! face.

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;

use crate::IronpagesError;

/// What a [`FontFallback::RegisteredOnly`] conversion does with a character
/// that none of the allowed fonts can draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MissingGlyphs {
    /// Paint nothing for the character. Text around it keeps its position.
    #[default]
    Blank,
    /// Fail the conversion with [`IronpagesError::FontError`], listing every
    /// character that had no glyph. No PDF bytes are written.
    Error,
}

/// Which fonts a conversion may draw with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontFallback {
    /// Registered fonts first, then host system fonts, the bundled Liberation
    /// and Noto faces, and platform CJK and emoji fonts.
    #[default]
    System,
    /// Draw only with fonts registered through
    /// [`add_font`](crate::HtmlConverter::add_font),
    /// [`add_font_variant`](crate::HtmlConverter::add_font_variant),
    /// [`add_font_pack`](crate::HtmlConverter::add_font_pack), or a CSS
    /// `@font-face` rule with a `url()` source.
    ///
    /// Generic and standard family names (`serif`, `sans-serif`, `monospace`,
    /// `Helvetica`, `Arial`, `Times`, `Courier`, ...) that are not registered
    /// draw with the PDF Base-14 fonts, which every PDF viewer provides, so no
    /// host font file is ever embedded. Characters the selected font cannot
    /// draw follow the [`MissingGlyphs`] policy.
    RegisteredOnly(MissingGlyphs),
}

impl FontFallback {
    /// Whether host, bundled, and platform fallback fonts may be loaded.
    pub(crate) const fn loads_unregistered_fonts(self) -> bool {
        matches!(self, Self::System)
    }
}

/// Weight and slant of one registered font face.
///
/// A family can register up to four faces. CSS `font-weight: bold` (or any
/// weight of 600 and above) selects a bold face and `font-style: italic` or
/// `oblique` selects an italic face; a missing face is synthesized from the
/// regular face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FontFaceStyle {
    /// The face is bold.
    pub bold: bool,
    /// The face is italic.
    pub italic: bool,
}

impl FontFaceStyle {
    /// The regular (upright, normal-weight) face.
    pub const REGULAR: Self = Self {
        bold: false,
        italic: false,
    };
    /// The bold face.
    pub const BOLD: Self = Self {
        bold: true,
        italic: false,
    };
    /// The italic face.
    pub const ITALIC: Self = Self {
        bold: false,
        italic: true,
    };
    /// The bold italic face.
    pub const BOLD_ITALIC: Self = Self {
        bold: true,
        italic: true,
    };

    /// Build a face style from a CSS numeric weight (`100`..=`900`) and an
    /// italic flag. Weights of 600 and above select the bold face.
    pub const fn from_css_weight(weight: u16, italic: bool) -> Self {
        Self {
            bold: weight >= 600,
            italic,
        }
    }

    /// Font-map key for `family` in this style.
    pub(crate) fn key(self, family: &str) -> String {
        crate::system_fonts::font_variant_key(family, self.bold, self.italic)
    }
}

thread_local! {
    static ACTIVE_FALLBACK: Cell<FontFallback> = const { Cell::new(FontFallback::System) };
    static MISSING_CHARACTERS: RefCell<BTreeSet<char>> = const { RefCell::new(BTreeSet::new()) };
}

/// Applies one conversion's [`FontFallback`] on the current thread and
/// collects the characters it could not draw. The previous policy and
/// collection are restored when the scope ends, so nested or reentrant
/// conversions stay independent.
pub(crate) struct FontFallbackScope {
    previous: FontFallback,
    previous_missing: BTreeSet<char>,
}

impl FontFallbackScope {
    pub(crate) fn enter(fallback: FontFallback) -> Self {
        let previous = ACTIVE_FALLBACK.with(|active| active.replace(fallback));
        let previous_missing = MISSING_CHARACTERS.with(|missing| missing.take());
        Self {
            previous,
            previous_missing,
        }
    }
}

impl Drop for FontFallbackScope {
    fn drop(&mut self) {
        ACTIVE_FALLBACK.with(|active| active.set(self.previous));
        MISSING_CHARACTERS
            .with(|missing| missing.replace(std::mem::take(&mut self.previous_missing)));
    }
}

fn active_fallback() -> FontFallback {
    ACTIVE_FALLBACK.with(Cell::get)
}

/// Whether the running conversion blanks characters its fonts cannot draw.
pub(crate) fn blanks_missing_glyphs() -> bool {
    matches!(active_fallback(), FontFallback::RegisteredOnly(_))
}

/// Characters that never need a visible glyph (Unicode default-ignorable code
/// points and C0/C1 controls); they are neither painted nor reported.
pub(crate) fn is_invisible(ch: char) -> bool {
    matches!(ch as u32,
        0x0000..=0x001F
        | 0x007F..=0x009F
        | 0x00AD
        | 0x034F
        | 0x061C
        | 0x115F..=0x1160
        | 0x17B4..=0x17B5
        | 0x180B..=0x180F
        | 0x200B..=0x200F
        | 0x202A..=0x202E
        | 0x2060..=0x206F
        | 0x3164
        | 0xFE00..=0xFE0F
        | 0xFEFF
        | 0xFFA0
        | 0xFFF0..=0xFFF8
        | 0x1BCA0..=0x1BCA3
        | 0x1D173..=0x1D17A
        | 0xE0000..=0xE0FFF
    )
}

/// Record characters painted without a glyph under
/// [`FontFallback::RegisteredOnly`].
pub(crate) fn record_missing(characters: impl IntoIterator<Item = char>) {
    if !blanks_missing_glyphs() {
        return;
    }
    MISSING_CHARACTERS.with(|missing| {
        missing
            .borrow_mut()
            .extend(characters.into_iter().filter(|ch| !is_invisible(*ch)));
    });
}

/// Decide whether a shaped glyph is painted. Under
/// [`FontFallback::RegisteredOnly`] the `.notdef` glyph (id 0) is never
/// painted: its characters are recorded and left blank.
pub(crate) fn paints_glyph(glyph_id: u16, unicode: &[u16]) -> bool {
    if glyph_id != 0 || !blanks_missing_glyphs() {
        return true;
    }
    record_missing(char::decode_utf16(unicode.iter().copied()).filter_map(Result::ok));
    false
}

/// Fail the conversion when [`MissingGlyphs::Error`] is active and some
/// characters had no glyph.
pub(crate) fn check_missing_glyphs() -> Result<(), IronpagesError> {
    if active_fallback() != FontFallback::RegisteredOnly(MissingGlyphs::Error) {
        return Ok(());
    }
    let missing = MISSING_CHARACTERS.with(|missing| missing.borrow().clone());
    if missing.is_empty() {
        return Ok(());
    }
    const SHOWN: usize = 20;
    let mut listed: Vec<String> = missing
        .iter()
        .take(SHOWN)
        .map(|ch| format!("U+{:04X} '{ch}'", *ch as u32))
        .collect();
    if missing.len() > SHOWN {
        listed.push(format!("and {} more", missing.len() - SHOWN));
    }
    Err(IronpagesError::FontError(format!(
        "no registered font has a glyph for {} character(s): {}. Register a font that covers them, \
         or use MissingGlyphs::Blank to leave them blank",
        missing.len(),
        listed.join(", ")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_restores_previous_policy_and_collection() {
        let outer = FontFallbackScope::enter(FontFallback::RegisteredOnly(MissingGlyphs::Error));
        record_missing(['中']);
        {
            let _inner = FontFallbackScope::enter(FontFallback::System);
            assert!(!blanks_missing_glyphs());
            record_missing(['文']);
            assert!(check_missing_glyphs().is_ok());
        }
        let error = check_missing_glyphs().unwrap_err().to_string();
        assert!(error.contains("U+4E2D '中'"), "{error}");
        assert!(!error.contains('文'), "{error}");
        drop(outer);
        assert!(!blanks_missing_glyphs());
    }

    #[test]
    fn invisible_characters_are_not_reported() {
        let _scope = FontFallbackScope::enter(FontFallback::RegisteredOnly(MissingGlyphs::Error));
        record_missing(['\u{200B}', '\u{FE0F}', '\n']);
        assert!(check_missing_glyphs().is_ok());
    }

    #[test]
    fn notdef_is_blank_only_under_registered_only() {
        assert!(paints_glyph(0, &[0x4E2D]));
        let _scope = FontFallbackScope::enter(FontFallback::RegisteredOnly(MissingGlyphs::Blank));
        assert!(!paints_glyph(0, &[0x4E2D]));
        assert!(paints_glyph(5, &[0x41]));
        assert!(check_missing_glyphs().is_ok());
    }

    #[test]
    fn css_weight_selects_bold_face_from_600() {
        assert_eq!(
            FontFaceStyle::from_css_weight(500, false),
            FontFaceStyle::REGULAR
        );
        assert_eq!(
            FontFaceStyle::from_css_weight(600, true),
            FontFaceStyle::BOLD_ITALIC
        );
        assert_eq!(FontFaceStyle::BOLD.key("Gfit Sans"), "gfit sans__bold");
    }
}
