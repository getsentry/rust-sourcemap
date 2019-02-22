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
#[macro_use]
extern crate serde_derive;

extern crate base64;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate url;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate if_chain;

mod macros;

pub use builder::SourceMapBuilder;
pub use decoder::{decode, decode_data_url, decode_slice};
pub use detector::{
    is_sourcemap, is_sourcemap_slice, locate_sourcemap_reference, locate_sourcemap_reference_slice,
    SourceMapRef,
};
pub use errors::{Error, Result};
pub use sourceview::SourceView;
pub use types::{
    DecodedMap, RawToken, RewriteOptions, SourceMap, SourceMapIndex, SourceMapSection,
    SourceMapSectionIter, Token, TokenIter,
};
pub use utils::make_relative_path;

mod builder;
mod decoder;
mod detector;
mod encoder;
mod errors;
mod jsontypes;
mod sourceview;
mod types;
mod utils;
mod vlq;

#[doc(hidden)]
pub mod internals {
    pub use super::decoder::StripHeaderReader;
    pub use vlq::{generate_vlq_segment, parse_vlq_segment};
}
