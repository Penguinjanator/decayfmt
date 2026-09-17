//! The decayfmt format definition: the binary header and the filename convention.
//!
//! This module owns two pieces of format metadata: the fixed 16-byte header (magic,
//! version, file type, and a per-type metadata region) and the filename convention that
//! carries the payload type and the instability value x (`name.idcy<x>` /
//! `name.tdcy<x>` / `name.adcy<x>`). It
//! knows nothing about corruption, file I/O, or the CLI. The header is written exactly
//! once at encode time and never mutated afterward; only the payload that follows it
//! ever changes. The invariant this module upholds is that a buffer is only accepted
//! as a header, and a name only accepted as a decayfmt name, if they match this build
//! exactly. Anything else is a typed refusal, never a guess.

use crate::error::DecayError;
use std::path::Path;

/// The four magic bytes that identify a decayfmt file: ASCII "DCYF".
pub const MAGIC: [u8; 4] = *b"DCYF";

/// The format version this build reads and writes. An unknown version is refused,
/// never interpreted, because the meaning of later versions is not knowable here.
///
/// Version 0x02 added the audio payload type. Files written by earlier builds carry
/// version 0x01 and are refused rather than reinterpreted: the metadata region is laid
/// out per payload type and the v1 layout differs.
pub const VERSION: u8 = 0x02;

/// file_type byte for an image payload (raw RGBA pixels).
pub const FILE_TYPE_IMAGE: u8 = 0x01;

/// file_type byte for a text payload (raw UTF-8 bytes).
pub const FILE_TYPE_TEXT: u8 = 0x02;

/// file_type byte for an audio payload (raw interleaved PCM samples).
pub const FILE_TYPE_AUDIO: u8 = 0x03;

/// Filename extension prefix that precedes x for an image, for example `idcy3`.
pub const IMAGE_EXTENSION_PREFIX: &str = "idcy";

/// Filename extension prefix that precedes x for text, for example `tdcy7`.
pub const TEXT_EXTENSION_PREFIX: &str = "tdcy";

/// Filename extension prefix that precedes x for audio, for example `adcy5`.
pub const AUDIO_EXTENSION_PREFIX: &str = "adcy";

/// Bits per sample of an audio payload. Sources are decoded to signed 16-bit
/// little-endian PCM regardless of their original depth, so every audio payload this
/// build writes carries this value. It is stored in the header rather than assumed so
/// a reader can reject a payload it would otherwise misinterpret.
pub const AUDIO_BITS_PER_SAMPLE: u8 = 16;

/// Bytes per audio sample, derived from [`AUDIO_BITS_PER_SAMPLE`].
pub const AUDIO_BYTES_PER_SAMPLE: usize = (AUDIO_BITS_PER_SAMPLE as usize) / 8;

/// Byte offset of the 4-byte little-endian image width within the header.
const WIDTH_OFFSET: usize = 6;

/// Byte offset of the 4-byte little-endian image height within the header.
const HEIGHT_OFFSET: usize = 10;

/// Byte offset of the 4-byte little-endian audio sample rate in hertz.
const SAMPLE_RATE_OFFSET: usize = 6;

/// Byte offset of the 1-byte audio channel count.
const CHANNELS_OFFSET: usize = 10;

/// Byte offset of the 1-byte audio bits-per-sample value.
const BITS_PER_SAMPLE_OFFSET: usize = 11;

/// Byte offset of the reserved region within the header.
const RESERVED_OFFSET: usize = 14;

/// Number of reserved bytes after the per-type metadata. Zero-filled on write,
/// ignored on read.
const RESERVED_LEN: usize = 2;

/// Total size of the fixed header: 4 (magic) + 1 (version) + 1 (file_type)
/// + 8 (per-type metadata) + 2 (reserved).
///
/// Bytes 6..14 are a per-type metadata region, interpreted according to the file_type
/// byte that precedes it: image stores width and height, audio stores sample rate,
/// channel count, and bits per sample, and text leaves the whole region zero.
pub const HEADER_SIZE: usize = RESERVED_OFFSET + RESERVED_LEN;

/// Which kind of payload follows the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Image,
    Text,
    Audio,
}

impl FileType {
    /// Maps a FileType to its on-disk byte.
    fn to_byte(self) -> u8 {
        match self {
            FileType::Image => FILE_TYPE_IMAGE,
            FileType::Text => FILE_TYPE_TEXT,
            FileType::Audio => FILE_TYPE_AUDIO,
        }
    }

    /// Maps an on-disk byte to a FileType, refusing any byte that is not a known
    /// file type rather than defaulting to one.
    fn from_byte(byte: u8) -> Result<FileType, DecayError> {
        match byte {
            FILE_TYPE_IMAGE => Ok(FileType::Image),
            FILE_TYPE_TEXT => Ok(FileType::Text),
            FILE_TYPE_AUDIO => Ok(FileType::Audio),
            other => Err(DecayError::UnsupportedFileType { found: other }),
        }
    }

    /// A short human-readable name for this file type, used in error messages when
    /// the filename's extension and the header disagree about the payload type.
    pub fn label(self) -> &'static str {
        match self {
            FileType::Image => "image",
            FileType::Text => "text",
            FileType::Audio => "audio",
        }
    }
}

/// Parses the decayfmt filename convention into the payload type and instability x.
///
/// The convention is `name.idcy<x>` for images, `name.tdcy<x>` for text, and
/// `name.adcy<x>` for audio, where x
/// is a positive integer. The payload type comes from the prefix and x from the
/// integer suffix. Both encode (to validate its output name) and open (to read x and
/// cross-check the type against the header) go through here, so the naming rule lives
/// in exactly one place. Every way a name can fail to fit the convention is a distinct
/// typed error: an unrecognized prefix, a missing or non-numeric x, a zero x, or an x
/// too large to fit a u32. x is never inferred from anywhere but the filename.
pub fn parse_filename(path: &Path) -> Result<(FileType, f64), DecayError> {
    let extension = path.extension().and_then(|raw| raw.to_str()).unwrap_or("");

    let (file_type, digits) = if let Some(rest) = extension.strip_prefix(IMAGE_EXTENSION_PREFIX) {
        (FileType::Image, rest)
    } else if let Some(rest) = extension.strip_prefix(TEXT_EXTENSION_PREFIX) {
        (FileType::Text, rest)
    } else if let Some(rest) = extension.strip_prefix(AUDIO_EXTENSION_PREFIX) {
        (FileType::Audio, rest)
    } else {
        return Err(DecayError::UnrecognizedExtension {
            extension: extension.to_string(),
        });
    };

    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(DecayError::FilenameNoX {
            filename: path.to_string_lossy().into_owned(),
        });
    }

    match digits.parse::<u32>() {
        Ok(0) => Err(DecayError::XNotPositive { value: 0.0 }),
        Ok(value) => Ok((file_type, f64::from(value))),
        Err(_) => Err(DecayError::XOutOfRange {
            value: digits.to_string(),
        }),
    }
}

/// The pixel dimensions of an image payload. Stored in the header so the flat RGBA
/// payload can be turned back into a viewable image when the file is opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// The playback parameters of an audio payload. Stored in the header so the flat
/// interleaved PCM payload can be turned back into a playable clip when the file is
/// opened. The payload is always signed 16-bit little-endian PCM, so `bits_per_sample`
/// records the format rather than selecting between several.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioSpec {
    pub sample_rate: u32,
    pub channels: u8,
    pub bits_per_sample: u8,
}

/// The per-type metadata carried in header bytes 6..14.
///
/// The region is interpreted according to the file_type byte that precedes it, so each
/// payload type names exactly the fields it needs and text carries none, so the
/// file_type byte fully determines the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metadata {
    Image(ImageDimensions),
    Text,
    Audio(AudioSpec),
}

/// The parsed, validated header of a decayfmt file. It carries the payload type and
/// the per-type metadata needed to interpret the raw payload that follows. Magic and
/// version are validated on read and not stored, because they are fixed for a given
/// build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub file_type: FileType,
    pub metadata: Metadata,
}

/// Reads a little-endian u32 from `buffer` at `offset`. The caller must have already
/// checked that the buffer is at least HEADER_SIZE bytes, so the four bytes are in range.
fn read_u32_le(buffer: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
}

impl Header {
    /// Builds a header for an image payload of the given pixel dimensions.
    pub fn for_image(width: u32, height: u32) -> Header {
        Header {
            file_type: FileType::Image,
            metadata: Metadata::Image(ImageDimensions { width, height }),
        }
    }

    /// Builds a header for a text payload, which carries no metadata.
    pub fn for_text() -> Header {
        Header {
            file_type: FileType::Text,
            metadata: Metadata::Text,
        }
    }

    /// Builds a header for an audio payload with the given playback parameters.
    pub fn for_audio(sample_rate: u32, channels: u8, bits_per_sample: u8) -> Header {
        Header {
            file_type: FileType::Audio,
            metadata: Metadata::Audio(AudioSpec {
                sample_rate,
                channels,
                bits_per_sample,
            }),
        }
    }

    /// Returns the image dimensions when this header describes an image.
    pub fn image_dimensions(&self) -> Option<ImageDimensions> {
        match self.metadata {
            Metadata::Image(dimensions) => Some(dimensions),
            _ => None,
        }
    }

    /// Returns the audio spec when this header describes an audio payload.
    pub fn audio_spec(&self) -> Option<AudioSpec> {
        match self.metadata {
            Metadata::Audio(spec) => Some(spec),
            _ => None,
        }
    }

    /// Serializes the header to its fixed 16-byte on-disk form.
    ///
    /// Upholds the invariant that the reserved bytes are always zero on write. The
    /// per-type metadata region is written according to the payload type: image
    /// dimensions as two little-endian u32 values, audio as a little-endian u32 sample
    /// rate followed by two single-byte fields, and text leaves the region zero. The
    /// header produced here is written once and never rewritten.
    pub fn write(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&MAGIC);
        bytes[4] = VERSION;
        bytes[5] = self.file_type.to_byte();
        match self.metadata {
            Metadata::Image(dimensions) => {
                bytes[WIDTH_OFFSET..WIDTH_OFFSET + 4]
                    .copy_from_slice(&dimensions.width.to_le_bytes());
                bytes[HEIGHT_OFFSET..HEIGHT_OFFSET + 4]
                    .copy_from_slice(&dimensions.height.to_le_bytes());
            }
            Metadata::Audio(spec) => {
                bytes[SAMPLE_RATE_OFFSET..SAMPLE_RATE_OFFSET + 4]
                    .copy_from_slice(&spec.sample_rate.to_le_bytes());
                bytes[CHANNELS_OFFSET] = spec.channels;
                bytes[BITS_PER_SAMPLE_OFFSET] = spec.bits_per_sample;
            }
            // For text the metadata bytes stay zero.
            Metadata::Text => {}
        }
        // The reserved bytes at bytes[RESERVED_OFFSET..] are always left zero.
        bytes
    }

    /// Parses and validates a header from the start of a buffer.
    ///
    /// Upholds the invariant that a header is only accepted if its magic and version
    /// match this build exactly. The per-type metadata region is read according to the
    /// file type. The trailing reserved bytes are ignored. Returns a typed error for
    /// every way the buffer can fail to be a header this build understands.
    pub fn read(buffer: &[u8]) -> Result<Header, DecayError> {
        if buffer.len() < HEADER_SIZE {
            return Err(DecayError::PayloadTooSmall {
                found: buffer.len(),
                needed: HEADER_SIZE,
            });
        }

        let mut found_magic = [0u8; 4];
        found_magic.copy_from_slice(&buffer[0..4]);
        if found_magic != MAGIC {
            return Err(DecayError::WrongMagic { found: found_magic });
        }

        let version = buffer[4];
        if version != VERSION {
            return Err(DecayError::UnsupportedVersion { found: version });
        }

        let file_type = FileType::from_byte(buffer[5])?;
        let metadata = match file_type {
            FileType::Image => Metadata::Image(ImageDimensions {
                width: read_u32_le(buffer, WIDTH_OFFSET),
                height: read_u32_le(buffer, HEIGHT_OFFSET),
            }),
            FileType::Text => Metadata::Text,
            FileType::Audio => Metadata::Audio(AudioSpec {
                sample_rate: read_u32_le(buffer, SAMPLE_RATE_OFFSET),
                channels: buffer[CHANNELS_OFFSET],
                bits_per_sample: buffer[BITS_PER_SAMPLE_OFFSET],
            }),
        };

        // The reserved bytes at buffer[RESERVED_OFFSET..HEADER_SIZE] are ignored.
        Ok(Header {
            file_type,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a valid image header buffer with known dimensions for tests to mutate.
    fn valid_image_header() -> Vec<u8> {
        Header::for_image(640, 480).write().to_vec()
    }

    #[test]
    fn headers_round_trip_with_their_own_metadata() {
        // write -> read is identity for each payload type, and the accessors return
        // metadata only for the type that carries it. The file_type byte is what
        // decides how the shared metadata region is read.
        let image = Header::for_image(640, 480);
        let text = Header::for_text();
        let audio = Header::for_audio(44_100, 2, AUDIO_BITS_PER_SAMPLE);

        for original in [image, text, audio] {
            let parsed = Header::read(&original.write()).expect("valid header must parse");
            assert_eq!(parsed, original, "round-trip changed the header");
        }

        let image = Header::read(&image.write()).expect("image header must parse");
        assert_eq!(
            image.image_dimensions(),
            Some(ImageDimensions {
                width: 640,
                height: 480
            })
        );
        assert!(
            image.audio_spec().is_none(),
            "an image carries no audio spec"
        );

        let text = Header::read(&text.write()).expect("text header must parse");
        assert!(
            text.image_dimensions().is_none(),
            "text carries no dimensions"
        );
        assert!(text.audio_spec().is_none(), "text carries no audio spec");

        let audio = Header::read(&audio.write()).expect("audio header must parse");
        assert_eq!(
            audio.audio_spec(),
            Some(AudioSpec {
                sample_rate: 44_100,
                channels: 2,
                bits_per_sample: AUDIO_BITS_PER_SAMPLE,
            })
        );
        assert!(
            audio.image_dimensions().is_none(),
            "audio carries no image dimensions"
        );
    }

    #[test]
    fn dimensions_are_written_little_endian() {
        let bytes = Header::for_image(0x0403_0201, 0x0807_0605).write();
        assert_eq!(
            &bytes[6..10],
            &[0x01, 0x02, 0x03, 0x04],
            "width must be little-endian"
        );
        assert_eq!(
            &bytes[10..14],
            &[0x05, 0x06, 0x07, 0x08],
            "height must be little-endian"
        );
    }

    #[test]
    fn reserved_bytes_are_zero_on_write() {
        // Even with dimensions set, the trailing reserved bytes must stay zero.
        let bytes = Header::for_image(640, 480).write();
        assert!(
            bytes[14..HEADER_SIZE].iter().all(|&b| b == 0),
            "reserved region must be zero-filled on write"
        );
    }

    #[test]
    fn magic_and_version_bytes_are_exact() {
        let bytes = Header::for_image(2, 2).write();
        assert_eq!(&bytes[0..4], b"DCYF", "magic bytes must be DCYF");
        assert_eq!(bytes[4], 0x02, "version byte must be 0x02");
        assert_eq!(bytes[5], FILE_TYPE_IMAGE, "file_type byte must be image");
    }

    #[test]
    fn wrong_magic_is_refused() {
        let mut bytes = valid_image_header();
        bytes[0] = b'X';
        match Header::read(&bytes) {
            Err(DecayError::WrongMagic { found }) => assert_eq!(found[0], b'X'),
            other => panic!("expected WrongMagic, got {:?}", other),
        }
    }

    #[test]
    fn unsupported_versions_are_refused() {
        // Both a future version and the v1 layout are refused rather than
        // reinterpreted: v1 laid out the metadata region differently, so a file from
        // an older build cannot be read with the current layout.
        for version in [0x01, 0x03] {
            let mut bytes = valid_image_header();
            bytes[4] = version;
            match Header::read(&bytes) {
                Err(DecayError::UnsupportedVersion { found }) => assert_eq!(found, version),
                other => panic!("expected UnsupportedVersion for 0x{version:02x}, got {other:?}"),
            }
        }
    }

    #[test]
    fn audio_sample_rate_is_written_little_endian() {
        let bytes = Header::for_audio(44_100, 2, AUDIO_BITS_PER_SAMPLE).write();
        assert_eq!(
            &bytes[6..10],
            &44_100u32.to_le_bytes(),
            "sample rate must be little-endian"
        );
        assert_eq!(bytes[10], 2, "channel count must follow the sample rate");
        assert_eq!(
            bytes[11], AUDIO_BITS_PER_SAMPLE,
            "bits per sample must follow the channel count"
        );
    }

    #[test]
    fn unknown_file_type_is_refused() {
        let mut bytes = valid_image_header();
        bytes[5] = 0x09;
        match Header::read(&bytes) {
            Err(DecayError::UnsupportedFileType { found }) => assert_eq!(found, 0x09),
            other => panic!("expected UnsupportedFileType, got {:?}", other),
        }
    }

    #[test]
    fn buffer_smaller_than_header_is_refused() {
        let short = [0u8; HEADER_SIZE - 1];
        match Header::read(&short) {
            Err(DecayError::PayloadTooSmall { found, needed }) => {
                assert_eq!(found, HEADER_SIZE - 1);
                assert_eq!(needed, HEADER_SIZE);
            }
            other => panic!("expected PayloadTooSmall, got {:?}", other),
        }
    }

    #[test]
    fn reserved_bytes_are_ignored_on_read() {
        // Only the trailing reserved bytes are ignored; flipping them must not stop
        // the header from parsing nor disturb the dimensions read before them.
        let mut bytes = valid_image_header();
        for b in bytes.iter_mut().take(HEADER_SIZE).skip(RESERVED_OFFSET) {
            *b = 0xFF;
        }
        let parsed = Header::read(&bytes).expect("reserved bytes must be ignored");
        assert_eq!(parsed.file_type, FileType::Image);
        assert_eq!(
            parsed.image_dimensions(),
            Some(ImageDimensions {
                width: 640,
                height: 480
            })
        );
    }

    #[test]
    fn parse_filename_reads_type_and_x() {
        for (name, expected) in [
            ("photo.idcy3", (FileType::Image, 3.0)),
            ("note.tdcy12", (FileType::Text, 12.0)),
            ("clip.adcy5", (FileType::Audio, 5.0)),
        ] {
            assert_eq!(
                parse_filename(Path::new(name)).expect("valid name must parse"),
                expected,
                "'{name}' did not parse to its type and x"
            );
        }
    }

    #[test]
    fn parse_filename_refuses_every_malformed_name() {
        // Each way a name can fail is a distinct error, so a caller can tell an
        // unknown extension from a missing x from an x out of range.
        for name in ["photo.png", "note.txt", "no_extension"] {
            assert!(
                matches!(
                    parse_filename(Path::new(name)),
                    Err(DecayError::UnrecognizedExtension { .. })
                ),
                "'{name}' should be an unrecognized extension"
            );
        }

        for name in ["photo.idcy", "note.tdcyx", "photo.idcy3a"] {
            assert!(
                matches!(
                    parse_filename(Path::new(name)),
                    Err(DecayError::FilenameNoX { .. })
                ),
                "'{name}' should yield FilenameNoX"
            );
        }

        assert!(matches!(
            parse_filename(Path::new("photo.idcy0")),
            Err(DecayError::XNotPositive { .. })
        ));

        // A run of digits that overflows u32 reports out-of-range, not "no x".
        assert!(matches!(
            parse_filename(Path::new("photo.idcy99999999999")),
            Err(DecayError::XOutOfRange { .. })
        ));
    }
}
