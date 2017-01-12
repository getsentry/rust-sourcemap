//! This library implements basic processing of JavaScript sourcemaps.
//!
//! ## Installation
//!
//! The crate is called sourcemap and you can depend on it via cargo:
//!
//! ```toml
//! [dependencies]
//! sourcemap = "*"
//! ```
//!
//! If you want to use the git version:
//!
//! ```toml
//! [dependencies.sourcemap]
//! git = "https://github.com/mitsuhiko/rust-sourcemap.git"
//! ```
//!
//! ## Basic Operation
//!
//! This crate can load JavaScript sourcemaps from JSON files.  It uses
//! `serde` for parsing of the JSON data.  Due to the nature of sourcemaps
//! the entirety of the file must be loaded into memory which can be quite
//! memory intensive.
//!
//! Usage:
//!
//! ```rust
//! use sourcemap::SourceMap;
//! let input: &[_] = b"{
//!     \"version\":3,
//!     \"sources\":[\"coolstuff.js\"],
//!     \"names\":[\"x\",\"alert\"],
//!     \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
//! }";
//! let sm = SourceMap::from_reader(input).unwrap();
//! let token = sm.lookup_token(0, 0).unwrap(); // line-number and column
//! println!("token: {}", token);
//! ```
#![cfg_attr(feature="serde_derive", feature(proc_macro))]

#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

extern crate base64;
extern crate serde;
extern crate serde_json;
extern crate url;

mod macros;

pub use types::{RawToken, Token, TokenIter, SourceMap, SourceMapIndex, SourceMapSection,
                SourceMapSectionIter, RewriteOptions, DecodedMap};
pub use builder::SourceMapBuilder;
pub use errors::{Result, Error};
pub use decoder::{decode, decode_slice, decode_data_url};
pub use detector::{SourceMapRef, locate_sourcemap_reference, locate_sourcemap_reference_slice,
                   is_sourcemap, is_sourcemap_slice};

mod builder;
mod errors;
mod types;
mod jsontypes;
mod decoder;
mod encoder;
mod detector;
mod vlq;
mod utils;

#[doc(hidden)]
pub mod internals {
    pub use super::decoder::StripHeaderReader;
    pub use vlq::{parse_vlq_segment, generate_vlq_segment};
}
