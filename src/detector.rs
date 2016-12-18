use std::str;
use std::io::{Read, BufRead, BufReader};

use errors::Result;
use types::DecodedMap;
use decoder::decode_data_url;


/// Represents a reference to a sourcemap
#[derive(PartialEq, Debug)]
pub enum SourceMapRef {
    /// A regular URL reference
    Ref(String),
    /// A legacy URL reference
    LegacyRef(String),
    /// Indicates a missing reference
    Missing
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
                return Ok(Some(try!(decode_data_url(url))));
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
        let line = try!(line);
        if line.starts_with("//# sourceMappingURL=") ||
           line.starts_with("//@ sourceMappingURL=") {
            let url = try!(str::from_utf8(&line.as_bytes()[21..])).trim().to_owned();
            if line.starts_with("//@") {
                return Ok(SourceMapRef::LegacyRef(url));
            } else {
                return Ok(SourceMapRef::Ref(url));
            }
        }
    }
    Ok(SourceMapRef::Missing)
}
