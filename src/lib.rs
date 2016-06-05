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
#![cfg_attr(feature="serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature="serde_macros", plugin(serde_macros))]

extern crate base64;
extern crate serde;
extern crate serde_json;

mod macros;

pub use types::{RawToken, Token, TokenIter, SourceMap, SourceMapIndex,
                SourceMapSection, SourceMapSectionIter};
pub use errors::{Result, Error};
pub use decoder::{decode, decode_data_url, DecodedMap};
pub use detector::{SourceMapRef, locate_sourcemap_reference};

mod errors;
mod types;
mod jsontypes;
mod decoder;
mod detector;

#[doc(hidden)]
pub mod internals {
    pub use super::decoder::StripHeaderReader;
}
