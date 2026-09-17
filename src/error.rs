//! All error types for decayfmt.
//!
//! Every error variant that any module can produce lives here, defined once as a
//! single complete error model. Each variant names the component, the operation,
//! and the condition that failed, so a failure is understandable without reading
//! the source. Errors are returned from library functions, never printed; the CLI
//! layer is responsible for printing them.

use std::fmt;

/// The four magic bytes every decayfmt file must begin with.
/// Repeated from format.rs intentionally so error messages can report what was
/// expected without depending on format.rs internals.
const EXPECTED_MAGIC: &[u8; 4] = b"DCYF";

/// The complete set of failures decayfmt can produce.
///
/// Every error any module returns is a variant here, each naming the component,
/// operation, and condition that failed.
#[derive(Debug)]
pub enum DecayError {
    /// The file did not start with the magic bytes DCYF. The file is not a
    /// decayfmt file and must not be parsed any further.
    WrongMagic { found: [u8; 4] },

    /// The version byte is not a version this build understands. An unknown version
    /// is refused; no forward compatibility is attempted.
    UnsupportedVersion { found: u8 },

    /// The file_type byte is not image (0x01), text (0x02), or audio (0x03).
    UnsupportedFileType { found: u8 },

    /// The filename's extension prefix and the header disagree about the payload
    /// type, for example an image file renamed to a `.tdcy<x>` name. The type cannot
    /// be determined, so the open is refused.
    MismatchedFileType {
        extension_kind: &'static str,
        header_kind: &'static str,
    },

    /// The target file is read-only. Corruption cannot be written, so the file is
    /// not displayed.
    ReadOnly { path: String },

    /// The buffer is shorter than the fixed 16-byte header, so no valid header
    /// could be read from it.
    PayloadTooSmall { found: usize, needed: usize },

    /// A text payload did not contain valid UTF-8 where valid UTF-8 was required.
    InvalidUtf8,

    /// The filename contained no parseable instability value x. x is read from the
    /// filename and nowhere else, so without it the file cannot be opened.
    FilenameNoX { filename: String },

    /// The instability value x parsed from the filename was not a positive number.
    XNotPositive { value: f64 },

    /// The instability value x in the filename was a run of digits too large to fit
    /// a u32.
    XOutOfRange { value: String },

    /// A filesystem read or write failed. The context names the component and the
    /// operation; the source is the underlying operating system error.
    Io {
        context: String,
        source: std::io::Error,
    },

    /// The source given to encode could not be decoded as a supported image. The
    /// context carries the underlying decoder message, kept as a string so this
    /// error type stays free of any image-crate dependency.
    ImageDecode { context: String },

    /// The source given to encode could not be decoded as a supported audio file.
    /// The context carries the underlying decoder message, kept as a string so this
    /// error type stays free of any audio-crate dependency.
    AudioDecode { context: String },

    /// On open, the corrupted audio could not be re-encoded for playback. The context
    /// carries the underlying encoder message as a string.
    AudioEncode { context: String },

    /// The output filename for encode had an extension that is not an image (idcy),
    /// text (tdcy), or audio (adcy) decayfmt extension, so the file type is unknown.
    UnrecognizedExtension { extension: String },

    /// On open, the image payload length did not match the width and height in the
    /// header, so the bytes cannot be interpreted as a complete image. The file is
    /// truncated or otherwise inconsistent with its own header.
    PayloadSizeMismatch { expected: usize, found: usize },

    /// On open, the corrupted image could not be re-encoded for display. The context
    /// carries the underlying encoder message as a string.
    ImageEncode { context: String },
}

impl fmt::Display for DecayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecayError::WrongMagic { found } => write!(
                f,
                "format: magic check failed: expected {:?} (DCYF), found {:?}. This is not a decayfmt file.",
                EXPECTED_MAGIC, found
            ),
            DecayError::UnsupportedVersion { found } => write!(
                f,
                "format: version check failed: version 0x{:02x} is not supported by this build. Refusing to guess at forward compatibility.",
                found
            ),
            DecayError::UnsupportedFileType { found } => write!(
                f,
                "format: file_type check failed: 0x{:02x} is not image (0x01), text (0x02), or audio (0x03).",
                found
            ),
            DecayError::MismatchedFileType {
                extension_kind,
                header_kind,
            } => write!(
                f,
                "open: type check failed: the filename extension indicates {} but the header says {}. Refusing to guess which is correct.",
                extension_kind, header_kind
            ),
            DecayError::ReadOnly { path } => write!(
                f,
                "open: writability check failed: '{}' is read-only. Corruption cannot be written, so the file will not be displayed.",
                path
            ),
            DecayError::PayloadTooSmall { found, needed } => write!(
                f,
                "format: header read failed: buffer is {} bytes but the header needs {} bytes.",
                found, needed
            ),
            DecayError::InvalidUtf8 => write!(
                f,
                "encode: text read failed: source is not valid UTF-8."
            ),
            DecayError::FilenameNoX { filename } => write!(
                f,
                "open: filename parse failed: '{}' contains no positive instability value x in its extension.",
                filename
            ),
            DecayError::XNotPositive { value } => write!(
                f,
                "open: instability check failed: x = {} is not a positive number.",
                value
            ),
            DecayError::XOutOfRange { value } => write!(
                f,
                "open: filename parse failed: instability value x = '{}' is too large; the maximum is {}.",
                value,
                u32::MAX
            ),
            DecayError::Io { context, source } => write!(f, "{}: {}", context, source),
            DecayError::ImageDecode { context } => write!(f, "{}", context),
            DecayError::AudioDecode { context } => write!(f, "{}", context),
            DecayError::AudioEncode { context } => write!(f, "{}", context),
            DecayError::UnrecognizedExtension { extension } => write!(
                f,
                "encode: output extension '{}' is not an image (idcy), text (tdcy), or audio (adcy) decayfmt extension.",
                extension
            ),
            DecayError::PayloadSizeMismatch { expected, found } => write!(
                f,
                "open: image payload check failed: header expects {} bytes of pixels but the payload is {} bytes. The file is truncated or inconsistent.",
                expected, found
            ),
            DecayError::ImageEncode { context } => write!(f, "{}", context),
        }
    }
}

impl std::error::Error for DecayError {
    /// Exposes the underlying operating system error for the Io variant so callers
    /// can inspect the cause chain. Other variants have no further source.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecayError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
