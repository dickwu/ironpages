//! How a PDF viewer presents the document when it is opened.
//!
//! These settings become the document catalog's `/PageLayout` and
//! `/OpenAction` entries (ISO 32000-1 §7.7.2 and §12.3.2.2). They do not
//! change page content.

/// Page arrangement a viewer uses when the document opens (`/PageLayout`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLayout {
    /// One page at a time.
    SinglePage,
    /// Pages in one continuous column.
    OneColumn,
    /// Pages in two continuous columns, odd pages on the left.
    TwoColumnLeft,
    /// Pages in two continuous columns, odd pages on the right.
    TwoColumnRight,
    /// Two pages at a time, odd pages on the left.
    TwoPageLeft,
    /// Two pages at a time, odd pages on the right.
    TwoPageRight,
}

impl PageLayout {
    /// `TwoPageLeft` and `TwoPageRight` were added in PDF 1.5.
    const fn requires_pdf_1_5(self) -> bool {
        matches!(self, Self::TwoPageLeft | Self::TwoPageRight)
    }

    const fn pdf_name(self) -> &'static str {
        match self {
            Self::SinglePage => "SinglePage",
            Self::OneColumn => "OneColumn",
            Self::TwoColumnLeft => "TwoColumnLeft",
            Self::TwoColumnRight => "TwoColumnRight",
            Self::TwoPageLeft => "TwoPageLeft",
            Self::TwoPageRight => "TwoPageRight",
        }
    }
}

/// Zoom a viewer applies to the first page when the document opens
/// (`/OpenAction` destination).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenZoom {
    /// Fit the whole page in the window (`/Fit`).
    FitPage,
    /// Fit the page width in the window (`/FitH`).
    FitWidth,
    /// Fit the page height in the window (`/FitV`).
    FitHeight,
    /// Show the page at a magnification in percent; `100.0` is actual size
    /// (`/XYZ`). Values are clamped to `1.0..=6400.0`.
    Percent(f32),
}

impl OpenZoom {
    /// Destination array on page object `page_id`. A `null` coordinate keeps
    /// the viewer's current position, which opens a document at its top.
    fn destination(self, page_id: usize) -> String {
        match self {
            Self::FitPage => format!("[{page_id} 0 R /Fit]"),
            Self::FitWidth => format!("[{page_id} 0 R /FitH null]"),
            Self::FitHeight => format!("[{page_id} 0 R /FitV null]"),
            Self::Percent(percent) => {
                let percent = if percent.is_finite() { percent } else { 100.0 };
                let scale = percent.clamp(1.0, 6400.0) / 100.0;
                format!(
                    "[{page_id} 0 R /XYZ null null {}]",
                    crate::render::pdf_syntax::format_pdf_number(scale)
                )
            }
        }
    }
}

/// Viewer presentation for one conversion. Both entries are omitted by
/// default, which leaves the choice to the viewer.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewerPreferences {
    /// Page arrangement when the document opens.
    pub page_layout: Option<PageLayout>,
    /// Zoom applied to the first page when the document opens.
    pub open_zoom: Option<OpenZoom>,
}

impl ViewerPreferences {
    /// Catalog entries (each with a leading space) for a document whose first
    /// page object id is `first_page`.
    pub(crate) fn catalog_entries(self, first_page: Option<usize>) -> String {
        let mut entries = String::new();
        if let Some(layout) = self.page_layout {
            entries.push_str(" /PageLayout /");
            entries.push_str(layout.pdf_name());
            if layout.requires_pdf_1_5() {
                // The file header says 1.4; the catalog raises the version.
                entries.push_str(" /Version /1.5");
            }
        }
        if let (Some(zoom), Some(page_id)) = (self.open_zoom, first_page) {
            entries.push_str(" /OpenAction ");
            entries.push_str(&zoom.destination(page_id));
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preferences_add_no_catalog_entries() {
        assert_eq!(ViewerPreferences::default().catalog_entries(Some(3)), "");
    }

    #[test]
    fn tcpdf_fullpage_single_page_display_mode() {
        let preferences = ViewerPreferences {
            page_layout: Some(PageLayout::SinglePage),
            open_zoom: Some(OpenZoom::FitPage),
        };
        assert_eq!(
            preferences.catalog_entries(Some(3)),
            " /PageLayout /SinglePage /OpenAction [3 0 R /Fit]"
        );
    }

    #[test]
    fn two_page_layouts_declare_pdf_1_5() {
        let preferences = ViewerPreferences {
            page_layout: Some(PageLayout::TwoPageRight),
            ..ViewerPreferences::default()
        };
        assert_eq!(
            preferences.catalog_entries(Some(3)),
            " /PageLayout /TwoPageRight /Version /1.5"
        );
        let preferences = ViewerPreferences {
            page_layout: Some(PageLayout::TwoColumnLeft),
            ..ViewerPreferences::default()
        };
        assert!(!preferences.catalog_entries(Some(3)).contains("/Version"));
    }

    #[test]
    fn percent_zoom_is_an_xyz_scale_at_the_page_top() {
        let preferences = ViewerPreferences {
            open_zoom: Some(OpenZoom::Percent(150.0)),
            ..ViewerPreferences::default()
        };
        assert_eq!(
            preferences.catalog_entries(Some(7)),
            " /OpenAction [7 0 R /XYZ null null 1.5]"
        );
        assert_eq!(
            OpenZoom::Percent(f32::NAN).destination(7),
            "[7 0 R /XYZ null null 1]"
        );
    }
}
