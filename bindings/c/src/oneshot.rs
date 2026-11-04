use ironpages_core::HtmlConverter;

use crate::handles::{IronpagesBuffer, IronpagesError, OutputSlot, boundary};
use crate::input::IronpagesBytes;
use crate::status::IronpagesStatus;

/// Convert UTF-8 HTML with a default one-shot converter.
///
/// # Safety
///
/// Input bytes and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_html_to_pdf(
    html: IronpagesBytes,
    out_pdf: *mut *mut IronpagesBuffer,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Input bytes and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let output = OutputSlot::required(out_pdf, "PDF")?;
            let html = html.parse_text("HTML")?;
            output.write(IronpagesBuffer::new(HtmlConverter::new().convert(html)?));
            Ok(())
        })
    }
}

/// Convert UTF-8 Markdown with a default one-shot converter.
///
/// # Safety
///
/// Input bytes and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_markdown_to_pdf(
    markdown: IronpagesBytes,
    out_pdf: *mut *mut IronpagesBuffer,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Input bytes and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let output = OutputSlot::required(out_pdf, "PDF")?;
            let markdown = markdown.parse_text("Markdown")?;
            output.write(IronpagesBuffer::new(
                HtmlConverter::new().convert_markdown(markdown)?,
            ));
            Ok(())
        })
    }
}
