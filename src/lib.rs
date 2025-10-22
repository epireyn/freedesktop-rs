#![warn(missing_docs)]
//! This crate is a Rust parser of Desktop Entry files.
//!
//! It follows the specification found on [their website](https://specifications.freedesktop.org/desktop-entry-spec/1.1/)
//!
//! # Usage
//! In this crate, everything is a [`DesktopFile`](crate::parser::models::DesktopFile) under the hood! It currently implements [`TryFrom<&[u8]>`] and [`TryFrom<&str>`], which means that you can easily parse strings and streams of bytes (e.g. a file).
//!
//! ```
//!  use std::fs::{self};
//!
//!  use freedesktop_rs::parser::models::DesktopFile;
//!
//!  fn parse_file(path: &str) -> Result<DesktopFile, nom::Err<nom::error::Error<Vec<u8>>>> {
//!      let content: Vec<u8> = fs::read(path).expect("File could not be read");
//!
//!      content.as_slice().try_into()
//! }
//! ```

/// Models and low level parser
pub mod parser;

/// Crate errors
pub mod error;

/// High level representations of specific Freedesktop structures
pub mod helpers;
