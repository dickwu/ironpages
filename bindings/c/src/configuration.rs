use crate::handles::{IronpagesConverter, IronpagesError, boundary};
use crate::status::{Failure, IRONPAGES_STATUS_INVALID_ARGUMENT, IronpagesStatus};

/// False value accepted by ABI boolean parameters.
pub const IRONPAGES_FALSE: u8 = 0;
/// True value accepted by ABI boolean parameters.
pub const IRONPAGES_TRUE: u8 = 1;

/// A fixed-width C boolean parsed into its semantic value.
struct CBoolean(bool);

impl CBoolean {
    /// Parse only the two documented ABI boolean values.
    fn parse(value: u8) -> Result<Self, Failure> {
        match value {
            IRONPAGES_FALSE => Ok(Self(false)),
            IRONPAGES_TRUE => Ok(Self(true)),
            _ => Err(Failure::new(
                crate::status::IRONPAGES_STATUS_INVALID_ENUM,
                format!(
                    "boolean value must be {IRONPAGES_FALSE} or {IRONPAGES_TRUE}, found {value}"
                ),
            )),
        }
    }
}

/// One finite quality value accepted by the native contract.
struct FiniteQuality(f32);

impl FiniteQuality {
    /// Reject values that cannot safely participate in numeric normalization.
    fn parse(value: f32, name: &str) -> Result<Self, Failure> {
        if !value.is_finite() {
            return Err(Failure::new(
                IRONPAGES_STATUS_INVALID_ARGUMENT,
                format!("{name} must be finite"),
            ));
        }
        Ok(Self(value))
    }
}

/// Enable or disable FlateDecode compression.
///
/// # Safety
///
/// Handles and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_converter_set_compress(
    converter: *mut IronpagesConverter,
    enabled: u8,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Raw handles and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let converter = IronpagesConverter::parse_mut(converter)?;
            let enabled = CBoolean::parse(enabled)?.0;
            converter.update(|current| current.compress(enabled));
            Ok(())
        })
    }
}

/// Set JPEG quality. Values above 100 are clamped by the renderer.
///
/// # Safety
///
/// Handles and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_converter_set_jpeg_quality(
    converter: *mut IronpagesConverter,
    quality: u8,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Raw handles and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let converter = IronpagesConverter::parse_mut(converter)?;
            converter.update(|current| current.jpeg_quality(quality));
            Ok(())
        })
    }
}

/// Enable or disable automatic source-image downscaling.
///
/// # Safety
///
/// Handles and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_converter_set_auto_resize_images(
    converter: *mut IronpagesConverter,
    enabled: u8,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Raw handles and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let converter = IronpagesConverter::parse_mut(converter)?;
            let enabled = CBoolean::parse(enabled)?.0;
            converter.update(|current| current.auto_resize_images(enabled));
            Ok(())
        })
    }
}

/// Set target source-image resolution in dots per inch.
///
/// # Safety
///
/// Handles and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_converter_set_image_dpi(
    converter: *mut IronpagesConverter,
    dpi: f32,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Raw handles and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let converter = IronpagesConverter::parse_mut(converter)?;
            let dpi = FiniteQuality::parse(dpi, "image DPI")?.0;
            converter.update(|current| current.image_dpi(dpi));
            Ok(())
        })
    }
}

/// Set CSS filter rasterization resolution in dots per inch.
///
/// # Safety
///
/// Handles and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_converter_set_filter_dpi(
    converter: *mut IronpagesConverter,
    dpi: f32,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Raw handles and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let converter = IronpagesConverter::parse_mut(converter)?;
            let dpi = FiniteQuality::parse(dpi, "filter DPI")?.0;
            converter.update(|current| current.filter_dpi(dpi));
            Ok(())
        })
    }
}

/// Set CSS mask rasterization resolution in dots per inch.
///
/// # Safety
///
/// Handles and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_converter_set_mask_dpi(
    converter: *mut IronpagesConverter,
    dpi: f32,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Raw handles and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let converter = IronpagesConverter::parse_mut(converter)?;
            let dpi = FiniteQuality::parse(dpi, "mask DPI")?.0;
            converter.update(|current| current.mask_dpi(dpi));
            Ok(())
        })
    }
}

/// Set flattened-background rasterization resolution in dots per inch.
///
/// # Safety
///
/// Handles and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_converter_set_background_raster_dpi(
    converter: *mut IronpagesConverter,
    dpi: f32,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Raw handles and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let converter = IronpagesConverter::parse_mut(converter)?;
            let dpi = FiniteQuality::parse(dpi, "background raster DPI")?.0;
            converter.update(|current| current.background_raster_dpi(dpi));
            Ok(())
        })
    }
}

/// Enable or disable conservative raster occlusion culling.
///
/// # Safety
///
/// Handles and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_converter_set_occlusion_cull(
    converter: *mut IronpagesConverter,
    enabled: u8,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Raw handles and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let converter = IronpagesConverter::parse_mut(converter)?;
            let enabled = CBoolean::parse(enabled)?.0;
            converter.update(|current| current.occlusion_cull(enabled));
            Ok(())
        })
    }
}

/// Enable or disable HTML sanitization.
///
/// # Safety
///
/// Handles and output pointers must follow `ABI.md`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ironpages_converter_set_sanitize(
    converter: *mut IronpagesConverter,
    enabled: u8,
    out_error: *mut *mut IronpagesError,
) -> IronpagesStatus {
    // SAFETY: Raw handles and output slots are validated before use.
    unsafe {
        boundary(out_error, || {
            let converter = IronpagesConverter::parse_mut(converter)?;
            let enabled = CBoolean::parse(enabled)?.0;
            converter.update(|current| current.sanitize(enabled));
            Ok(())
        })
    }
}
