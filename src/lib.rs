//! decayfmt: a file format where decay is a first-class property.
//!
//! Every open corrupts the payload on disk by an amount taken from the filename.
//! The crate exposes the whole format: the typed error model ([`error`]), the binary
//! header and filename convention ([`format`]), the corruption algorithm ([`corrupt`]),
//! and the two flows built on them, [`encode`] and [`open`]. Images, text, and audio
//! are supported.

pub mod corrupt;
pub mod encode;
pub mod error;
pub mod format;
pub mod open;
