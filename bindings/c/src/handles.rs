use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

use crate::status::{
    Failure, IRONPAGES_STATUS_INTERNAL, IRONPAGES_STATUS_INVALID_ARGUMENT,
    IRONPAGES_STATUS_INVALID_HANDLE, IRONPAGES_STATUS_OK, IRONPAGES_STATUS_OUTPUT_NOT_EMPTY,
    IronpagesStatus,
};

/// Opaque owner of one configured Ironpages converter.
pub struct IronpagesConverter {
    pub(crate) converter: ironpages_core::HtmlConverter,
}

impl IronpagesConverter {
    /// Borrow a live converter handle for one read-only foreign call.
    ///
    /// # Safety
    ///
    /// A non-null pointer must come from `ironpages_converter_new`, remain
    /// alive for the borrow, and not be mutated concurrently.
    pub(crate) unsafe fn parse_ref<'a>(raw: *const Self) -> Result<&'a Self, Failure> {
        if raw.is_null() {
            return Err(Failure::new(
                IRONPAGES_STATUS_INVALID_HANDLE,
                "converter handle is null",
            ));
        }
        // SAFETY: The foreign-call contract above requires a live shared handle.
        Ok(unsafe { &*raw })
    }

    /// Borrow a live converter handle for one foreign call.
    ///
    /// # Safety
    ///
    /// A non-null pointer must come from `ironpages_converter_new`, remain
    /// exclusively accessible for the borrow, and not have been freed.
    pub(crate) unsafe fn parse_mut<'a>(raw: *mut Self) -> Result<&'a mut Self, Failure> {
        if raw.is_null() {
            return Err(Failure::new(
                IRONPAGES_STATUS_INVALID_HANDLE,
                "converter handle is null",
            ));
        }
        // SAFETY: The foreign-call contract above requires a live unique handle.
        Ok(unsafe { &mut *raw })
    }

    /// Replace the immutable builder value while preserving handle identity.
    pub(crate) fn update(
        &mut self,
        configure: impl FnOnce(ironpages_core::HtmlConverter) -> ironpages_core::HtmlConverter,
    ) {
        self.converter = configure(std::mem::take(&mut self.converter));
    }
}

/// Opaque owner of PDF bytes allocated by Ironpages.
pub struct IronpagesBuffer {
    bytes: Vec<u8>,
}

impl IronpagesBuffer {
    /// Own one completed PDF allocation.
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

/// Opaque owner of one categorized failure and its UTF-8 diagnostic.
pub struct IronpagesError {
    status: IronpagesStatus,
    message: String,
}

impl From<Failure> for IronpagesError {
    fn from(failure: Failure) -> Self {
        let status = failure.status();
        Self {
            status,
            message: failure.into_message(),
        }
    }
}

/// A caller-owned pointer slot proven empty before an allocation is installed.
pub(crate) struct OutputSlot<'a, T> {
    target: &'a mut *mut T,
}

impl<'a, T> OutputSlot<'a, T> {
    /// Parse a required output slot once at the foreign boundary.
    ///
    /// # Safety
    ///
    /// `raw` must be null or point to a writable, exclusively borrowed `*mut T`.
    pub(crate) unsafe fn required(raw: *mut *mut T, name: &str) -> Result<Self, Failure> {
        if raw.is_null() {
            return Err(Failure::new(
                IRONPAGES_STATUS_INVALID_ARGUMENT,
                format!("{name} output slot is null"),
            ));
        }
        // SAFETY: The foreign-call contract above requires a writable unique slot.
        let target = unsafe { &mut *raw };
        if !target.is_null() {
            return Err(Failure::new(
                IRONPAGES_STATUS_OUTPUT_NOT_EMPTY,
                format!("{name} output slot must be initialized to null"),
            ));
        }
        Ok(Self { target })
    }

    /// Install one allocation and transfer its ownership to the caller.
    pub(crate) fn write(self, value: T) {
        *self.target = Box::into_raw(Box::new(value));
    }
}

/// Execute one operation without allowing Rust unwinding to cross the ABI.
///
/// # Safety
///
/// A non-null error slot must satisfy [`OutputSlot::required`].
pub(crate) unsafe fn boundary(
    out_error: *mut *mut IronpagesError,
    operation: impl FnOnce() -> Result<(), Failure>,
) -> IronpagesStatus {
    let error_slot = if out_error.is_null() {
        None
    } else {
        // SAFETY: The function inherits the slot contract documented above.
        match unsafe { OutputSlot::required(out_error, "error") } {
            Ok(slot) => Some(slot),
            Err(failure) => return failure.status(),
        }
    };

    let failure = match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => return IRONPAGES_STATUS_OK,
        Ok(Err(failure)) => failure,
        Err(_) => Failure::new(
            IRONPAGES_STATUS_INTERNAL,
            "Ironpages caught an unexpected internal panic",
        ),
    };
    let status = failure.status();
    if let Some(slot) = error_slot {
        slot.write(failure.into());
    }
    status
}

/// Release an opaque owned handle and clear the caller's slot.
///
/// # Safety
///
/// `raw` must be null or point to a writable handle slot. A non-null handle in
/// that slot must be the unique owner returned by Ironpages.
pub(crate) unsafe fn free_owned<T>(raw: *mut *mut T) -> IronpagesStatus {
    if raw.is_null() {
        return IRONPAGES_STATUS_INVALID_ARGUMENT;
    }
    let released = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: The foreign-call contract above requires a writable unique slot.
        let target = unsafe { &mut *raw };
        let owned = std::mem::replace(target, ptr::null_mut());
        if !owned.is_null() {
            // SAFETY: A non-null slot is the unique Box owner returned by Ironpages.
            drop(unsafe { Box::from_raw(owned) });
        }
    }));
    match released {
        Ok(()) => IRONPAGES_STATUS_OK,
        Err(_) => IRONPAGES_STATUS_INTERNAL,
    }
}

/// Return a borrowed pointer to the first PDF byte, or null for a null handle.
///
/// # Safety
///
/// A non-null `buffer` must identify a live buffer owned by the caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_buffer_data(buffer: *const IronpagesBuffer) -> *const u8 {
    // SAFETY: `as_ref` only borrows a caller-provided handle when it is non-null.
    unsafe { buffer.as_ref() }.map_or(ptr::null(), |buffer| buffer.bytes.as_ptr())
}

/// Return the PDF byte length, or zero for a null handle.
///
/// # Safety
///
/// A non-null `buffer` must identify a live buffer owned by the caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_buffer_len(buffer: *const IronpagesBuffer) -> usize {
    // SAFETY: `as_ref` only borrows a caller-provided handle when it is non-null.
    unsafe { buffer.as_ref() }.map_or(0, |buffer| buffer.bytes.len())
}

/// Release a PDF buffer and clear its owning handle.
///
/// # Safety
///
/// `buffer` must be null or identify a writable owning slot. A non-null value
/// in that slot must be the unique buffer owner returned by Ironpages.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_buffer_free(
    buffer: *mut *mut IronpagesBuffer,
) -> IronpagesStatus {
    // SAFETY: The caller accepts the ownership contract above.
    unsafe { free_owned(buffer) }
}

/// Return the status stored in an error handle.
///
/// # Safety
///
/// A non-null `error` must identify a live error owned by the caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_error_status(error: *const IronpagesError) -> IronpagesStatus {
    // SAFETY: `as_ref` only borrows a caller-provided handle when it is non-null.
    unsafe { error.as_ref() }.map_or(IRONPAGES_STATUS_INVALID_HANDLE, |error| error.status)
}

/// Return a borrowed pointer to the first error-message byte.
///
/// # Safety
///
/// A non-null `error` must identify a live error owned by the caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_error_message_data(error: *const IronpagesError) -> *const u8 {
    // SAFETY: `as_ref` only borrows a caller-provided handle when it is non-null.
    unsafe { error.as_ref() }.map_or(ptr::null(), |error| error.message.as_ptr())
}

/// Return the UTF-8 error-message length, or zero for a null handle.
///
/// # Safety
///
/// A non-null `error` must identify a live error owned by the caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_error_message_len(error: *const IronpagesError) -> usize {
    // SAFETY: `as_ref` only borrows a caller-provided handle when it is non-null.
    unsafe { error.as_ref() }.map_or(0, |error| error.message.len())
}

/// Release an error and clear its owning handle.
///
/// # Safety
///
/// `error` must be null or identify a writable owning slot. A non-null value in
/// that slot must be the unique error owner returned by Ironpages.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_error_free(error: *mut *mut IronpagesError) -> IronpagesStatus {
    // SAFETY: The caller accepts the ownership contract above.
    unsafe { free_owned(error) }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::*;

    #[test]
    fn boundary_turns_a_panic_into_an_owned_internal_error() {
        let mut error = ptr::null_mut();

        // SAFETY: The test provides one initialized writable error slot.
        let status = unsafe {
            boundary(&mut error, || -> Result<(), Failure> {
                panic!("foreign boundary regression")
            })
        };

        assert_eq!(status, IRONPAGES_STATUS_INTERNAL);
        assert!(!error.is_null());
        // SAFETY: `error` remains alive until the matching free call below.
        assert_eq!(
            unsafe { ironpages_error_status(error) },
            IRONPAGES_STATUS_INTERNAL
        );
        // SAFETY: `error` remains alive until the matching free call below.
        assert!(unsafe { ironpages_error_message_len(error) } > 0);
        // SAFETY: `error` is the unique owner returned by `boundary`.
        assert_eq!(
            unsafe { ironpages_error_free(&mut error) },
            IRONPAGES_STATUS_OK
        );
        assert!(error.is_null());
    }

    #[test]
    fn opaque_owners_can_move_between_threads_while_idle() {
        fn assert_send<T: Send>() {}

        assert_send::<IronpagesConverter>();
        assert_send::<IronpagesBuffer>();
        assert_send::<IronpagesError>();
    }
}
