use std::io::{BufRead, BufReader, Read};
use std::str;

use crate::decoder::{decode_data_url, strip_junk_header, StripHeaderReader};
use crate::errors::Result;
use crate::jsontypes::MinimalRawSourceMap;
use crate::types::DecodedMap;

use serde_json;

/// Represents a reference to a sourcemap
#[derive(PartialEq, Debug)]
pub enum SourceMapRef {
    /// A regular URL reference
    Ref(String),
    /// A legacy URL reference
    LegacyRef(String),
    /// Indicates a missing reference
    Missing,
}

impl SourceMapRef {
    /// Return the URL of the reference
    pub fn get_url(&self) -> Option<&str> {
        match *self {
            SourceMapRef::Ref(ref u) => Some(u.as_str()),
            SourceMapRef::LegacyRef(ref u) => Some(u.as_str()),
            SourceMapRef::Missing => None,
        }
    }

    /// Load an embedded sourcemap if there is a data URL.
    pub fn get_embedded_sourcemap(&self) -> Result<Option<DecodedMap>> {
        if let Some(url) = self.get_url() {
            if url.starts_with("data:") {
                return Ok(Some(decode_data_url(url)?));
            }
        }
        Ok(None)
    }
}

/// Locates a sourcemap reference
///
/// Given a reader to a JavaScript file this tries to find the correct
/// sourcemap reference comment and return it.
pub fn locate_sourcemap_reference<R: Read>(rdr: R) -> Result<SourceMapRef> {
    for line in BufReader::new(rdr).lines() {
        let line = line?;
        if line.starts_with("//# sourceMappingURL=") || line.starts_with("//@ sourceMappingURL=") {
            let url = str::from_utf8(&line.as_bytes()[21..])?.trim().to_owned();
            if line.starts_with("//@") {
                return Ok(SourceMapRef::LegacyRef(url));
            } else {
                return Ok(SourceMapRef::Ref(url));
            }
        }
    }
    Ok(SourceMapRef::Missing)
}

/// Locates a sourcemap reference in a slice
///
/// This is an alternative to `locate_sourcemap_reference` that operates
/// on slices.
pub fn locate_sourcemap_reference_slice(slice: &[u8]) -> Result<SourceMapRef> {
    locate_sourcemap_reference(slice)
}

fn is_sourcemap_common(rsm: MinimalRawSourceMap) -> bool {
    (rsm.version.is_some() || rsm.file.is_some())
        && ((rsm.sources.is_some()
            || rsm.source_root.is_some()
            || rsm.sources_content.is_some()
            || rsm.names.is_some())
            && rsm.mappings.is_some())
        || rsm.sections.is_some()
}

fn is_sourcemap_impl<R: Read>(rdr: R) -> Result<bool> {
    let mut rdr = StripHeaderReader::new(rdr);
    let mut rdr = BufReader::new(&mut rdr);
    let rsm: MinimalRawSourceMap = serde_json::from_reader(&mut rdr)?;
    Ok(is_sourcemap_common(rsm))
}

fn is_sourcemap_slice_impl(slice: &[u8]) -> Result<bool> {
    let content = strip_junk_header(slice)?;
    let rsm: MinimalRawSourceMap = serde_json::from_slice(content)?;
    Ok(is_sourcemap_common(rsm))
}

/// Checks if a valid sourcemap can be read from the given reader
pub fn is_sourcemap<R: Read>(rdr: R) -> bool {
    is_sourcemap_impl(rdr).unwrap_or(false)
}

/// Checks if the given byte slice contains a sourcemap
pub fn is_sourcemap_slice(slice: &[u8]) -> bool {
    is_sourcemap_slice_impl(slice).unwrap_or(false)
}
