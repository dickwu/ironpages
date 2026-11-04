use ironpages_core::IronpagesError;

/// Machine-readable result returned by every fallible C operation.
pub type IronpagesStatus = i32;

/// The operation completed successfully.
pub const IRONPAGES_STATUS_OK: IronpagesStatus = 0;
/// A pointer, length, number, or output slot violates the function contract.
pub const IRONPAGES_STATUS_INVALID_ARGUMENT: IronpagesStatus = 1;
/// Text input is not valid UTF-8.
pub const IRONPAGES_STATUS_INVALID_UTF8: IronpagesStatus = 2;
/// A fixed-width integer does not identify a documented ABI value.
pub const IRONPAGES_STATUS_INVALID_ENUM: IronpagesStatus = 3;
/// A required opaque handle is null.
pub const IRONPAGES_STATUS_INVALID_HANDLE: IronpagesStatus = 4;
/// An output slot was not initialized to null by the caller.
pub const IRONPAGES_STATUS_OUTPUT_NOT_EMPTY: IronpagesStatus = 5;
/// The HTML parser rejected the document.
pub const IRONPAGES_STATUS_PARSE: IronpagesStatus = 10;
/// The CSS parser rejected the document.
pub const IRONPAGES_STATUS_CSS: IronpagesStatus = 11;
/// The layout engine could not lay out the document.
pub const IRONPAGES_STATUS_LAYOUT: IronpagesStatus = 12;
/// The PDF renderer could not produce the document.
pub const IRONPAGES_STATUS_RENDER: IronpagesStatus = 13;
/// A font or font pack could not be parsed or embedded.
pub const IRONPAGES_STATUS_FONT: IronpagesStatus = 14;
/// A filesystem operation failed.
pub const IRONPAGES_STATUS_IO: IronpagesStatus = 15;
/// The security policy rejected the document.
pub const IRONPAGES_STATUS_SECURITY: IronpagesStatus = 16;
/// Ironpages caught an unexpected internal panic at the foreign boundary.
pub const IRONPAGES_STATUS_INTERNAL: IronpagesStatus = 255;

/// One categorized failure ready to cross the C boundary.
#[derive(Debug)]
pub(crate) struct Failure {
    status: IronpagesStatus,
    message: String,
}

impl Failure {
    /// Create a failure from its stable category and diagnostic.
    pub(crate) fn new(status: IronpagesStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    /// Return the stable machine-readable category.
    pub(crate) const fn status(&self) -> IronpagesStatus {
        self.status
    }

    /// Consume the failure and return its diagnostic.
    pub(crate) fn into_message(self) -> String {
        self.message
    }
}

impl From<IronpagesError> for Failure {
    fn from(error: IronpagesError) -> Self {
        let status = match &error {
            IronpagesError::ParseError(_) => IRONPAGES_STATUS_PARSE,
            IronpagesError::CssError(_) => IRONPAGES_STATUS_CSS,
            IronpagesError::LayoutError(_) => IRONPAGES_STATUS_LAYOUT,
            IronpagesError::RenderError(_) => IRONPAGES_STATUS_RENDER,
            IronpagesError::FontError(_) => IRONPAGES_STATUS_FONT,
            IronpagesError::IoError(_) => IRONPAGES_STATUS_IO,
            IronpagesError::SecurityError(_) => IRONPAGES_STATUS_SECURITY,
        };
        Self::new(status, error.to_string())
    }
}

impl From<ironpages_core::FontPackError> for Failure {
    fn from(error: ironpages_core::FontPackError) -> Self {
        Self::new(IRONPAGES_STATUS_FONT, error.to_string())
    }
}
