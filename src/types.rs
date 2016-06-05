use std::fmt;
use std::io::Read;

use decoder::{decode, DecodedMap};
use errors::{Result, Error};

/// Represents a raw token
///
/// Raw tokens are used internally to represent the sourcemap
/// in a memory efficient way.  If you construct sourcemaps yourself
/// then you need to create these objects, otherwise they are invisible
/// to you as a user.
#[derive(PartialEq, Copy, Clone)]
pub struct RawToken {
    /// the destination (minified) line number
    pub dst_line: u32,
    /// the destination (minified) column number
    pub dst_col: u32,
    /// the source line number
    pub src_line: u32,
    /// the source line column
    pub src_col: u32,
    /// source identifier
    pub src_id: u32,
    /// name identifier (`!0` in case there is no associated name)
    pub name_id: u32,
}

/// Represents a token from a sourcemap
pub struct Token<'a> {
    raw: &'a RawToken,
    i: &'a SourceMap,
}

impl<'a> Token<'a> {
    /// get the destination (minified) line number
    pub fn get_dst_line(&self) -> u32 {
        self.raw.dst_line
    }

    /// get the destination (minified) column number
    pub fn get_dst_col(&self) -> u32 {
        self.raw.dst_col
    }

    /// get the destination line and column
    pub fn get_dst(&self) -> (u32, u32) {
        (self.get_dst_line(), self.get_dst_col())
    }

    /// get the source line number
    pub fn get_src_line(&self) -> u32 {
        self.raw.src_line
    }

    /// get the source column number
    pub fn get_src_col(&self) -> u32 {
        self.raw.src_col
    }

    /// get the source line and column
    pub fn get_src(&self) -> (u32, u32) {
        (self.get_src_line(), self.get_src_col())
    }

    /// get the source if it exists as string
    pub fn get_source(&self) -> &str {
        if self.raw.src_id == !0 {
            ""
        } else {
            &self.i.sources[self.raw.src_id as usize][..]
        }
    }

    /// get the name if it exists as string
    pub fn get_name(&self) -> Option<&str> {
        if self.raw.name_id == !0 {
            None
        } else {
            Some(&self.i.names[self.raw.name_id as usize][..])
        }
    }

    /// returns `true` if a name exists, `false` otherwise
    pub fn has_name(&self) -> bool {
        self.get_name().is_some()
    }

    /// Converts the token into a debug tuple in the form
    /// `(source, src_line, src_col, name)`
    pub fn to_tuple(&self) -> (&str, u32, u32, Option<&str>) {
        (
            self.get_source(),
            self.get_src_line(),
            self.get_src_col(),
            self.get_name()
        )
    }

    /// Get the underlying raw token
    pub fn get_raw_token(&self) -> RawToken {
        *self.raw
    }
}

/// Iterates over all tokens in a sourcemap
pub struct TokenIter<'a> {
    i: &'a SourceMap,
    next_idx: u32,
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        self.i.get_token(self.next_idx).map(|tok| {
            self.next_idx += 1;
            tok
        })
    }
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Token {}>", self)
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}{}",
               self.get_source(),
               self.get_src_line(),
               self.get_src_col(),
               self.get_name().map(|x| format!(" name={}", x))
                   .unwrap_or("".into()))
    }
}

/// Represents a section in a sourcemap index
pub struct SourceMapSection {
    offset: (u32, u32),
    url: Option<String>,
    map: Option<Box<SourceMap>>,
}

/// Iterates over all sections in a sourcemap index
pub struct SourceMapSectionIter<'a> {
    i: &'a SourceMapIndex,
    next_idx: u32,
}

impl<'a> Iterator for SourceMapSectionIter<'a> {
    type Item = &'a SourceMapSection;

    fn next(&mut self) -> Option<&'a SourceMapSection> {
        self.i.get_section(self.next_idx).map(|sec| {
            self.next_idx += 1;
            sec
        })
    }
}

/// Represents a sourcemap index in memory
pub struct SourceMapIndex {
    version: u32,
    file: Option<String>,
    sections: Vec<SourceMapSection>,
}

/// Represents a sourcemap in memory
///
/// This is always represents a regular "non-indexed" sourcemap.  Particularly
/// in case the `from_reader` method is used an index sourcemap will be
/// rejected with an error on reading.
pub struct SourceMap {
    version: u32,
    file: Option<String>,
    tokens: Vec<RawToken>,
    index: Vec<(u32, u32, u32)>,
    names: Vec<String>,
    sources: Vec<String>,
}

impl SourceMap {

    /// Creates a sourcemap from a reader over a JSON stream in UTF-8
    /// format.  Optionally a "garbage header" as defined by the
    /// sourcemap draft specification is supported.  In case an indexed
    /// sourcemap is encountered an error is returned.
    ///
    /// ```rust
    /// use sourcemap::SourceMap;
    /// let input: &[_] = b"{
    ///     \"version\":3,
    ///     \"sources\":[\"coolstuff.js\"],
    ///     \"names\":[\"x\",\"alert\"],
    ///     \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
    /// }";
    /// let sm = SourceMap::from_reader(input).unwrap();
    /// ```
    pub fn from_reader<R: Read>(rdr: R) -> Result<SourceMap> {
        match try!(decode(rdr)) {
            DecodedMap::Regular(sm) => Ok(sm),
            DecodedMap::Index(_) => Err(Error::IndexedSourcemap),
        }
    }

    /// Constructs a new sourcemap from raw components.
    ///
    /// - `version`: the version is typically `3` which is the current darft version
    /// - `file`: an optional filename of the sourcemap
    /// - `tokens`: a list of raw tokens
    /// - `index`: a sorted mapping of line and column to token index
    /// - `names`: a vector of names
    /// - `sources` a vector of source filenames
    pub fn new(version: u32, file: Option<String>, tokens: Vec<RawToken>,
               index: Vec<(u32, u32, u32)>, names: Vec<String>,
               sources: Vec<String>) -> SourceMap {
        SourceMap {
            version: version,
            file: file,
            tokens: tokens,
            index: index,
            names: names,
            sources: sources,
        }
    }

    /// Returns the version of the sourcemap.
    pub fn get_version(&self) -> u32 {
        self.version
    }

    /// Returns the embedded filename in case there is one.
    pub fn get_file(&self) -> Option<&str> {
        self.file.as_ref().map(|x| &x[..])
    }

    /// Looks up a token by its index.
    pub fn get_token<'a>(&'a self, idx: u32) -> Option<Token<'a>> {
        self.tokens.get(idx as usize).map(|raw| {
            Token { raw: raw, i: self }
        })
    }

    /// Returns the number of tokens in the sourcemap.
    pub fn get_token_count(&self) -> u32 {
        self.tokens.len() as u32
    }

    /// Returns an iterator over the tokens.
    pub fn tokens<'a>(&'a self) -> TokenIter<'a> {
        TokenIter { i: self, next_idx: 0 }
    }

    /// Looks up the closest token to a given line and column.
    pub fn lookup_token<'a>(&'a self, line: u32, col: u32) -> Option<Token<'a>> {
        let mut low = 0;
        let mut high = self.index.len();

        while low < high {
            let mid = (low + high) / 2;
            if (line, col, 0) < self.index[mid as usize] {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        if low > 0 {
            self.get_token(self.index[low as usize].2)
        } else {
            None
        }
    }

    /// Returns the number of sources in the sourcemap.
    pub fn get_source_count(&self) -> u32 {
        self.sources.len() as u32
    }

    /// Looks up a source for a specific index.
    pub fn get_source(&self, idx: u32) -> Option<&str> {
        self.sources.get(idx as usize).map(|x| &x[..])
    }

    /// Returns the number of names in the sourcemap.
    pub fn get_name_count(&self) -> u32 {
        self.names.len() as u32
    }

    /// Looks up a name for a specific index.
    pub fn get_name(&self, idx: u32) -> Option<&str> {
        self.names.get(idx as usize).map(|x| &x[..])
    }
}

impl SourceMapIndex {

    /// Creates a sourcemap index from a reader over a JSON stream in UTF-8
    /// format.  Optionally a "garbage header" as defined by the
    /// sourcemap draft specification is supported.  In case a regular
    /// sourcemap is encountered an error is returned.
    pub fn from_reader<R: Read>(rdr: R) -> Result<SourceMapIndex> {
        match try!(decode(rdr)) {
            DecodedMap::Regular(_) => Err(Error::RegularSourcemap),
            DecodedMap::Index(smi) => Ok(smi),
        }
    }

    /// Constructs a new sourcemap index from raw components.
    ///
    /// - `version`: the version is typically `3` which is the current darft version
    /// - `file`: an optional filename of the index
    /// - `sections`: a vector of source map index sections
    pub fn new(version: u32, file: Option<String>,
               sections: Vec<SourceMapSection>) -> SourceMapIndex {
        SourceMapIndex {
            version: version,
            file: file,
            sections: sections,
        }
    }

    /// Returns the version of the sourcemap index.
    pub fn get_version(&self) -> u32 {
        self.version
    }

    /// Returns the embedded filename in case there is one.
    pub fn get_file(&self) -> Option<&str> {
        self.file.as_ref().map(|x| &x[..])
    }

    /// Returns the number of sections in this index
    pub fn get_section_count(&self) -> u32 {
        self.sections.len() as u32
    }

    /// Looks up a single section and returns it
    pub fn get_section(&self, idx: u32) -> Option<&SourceMapSection> {
        self.sections.get(idx as usize)
    }

    /// Looks up a single section and returns it as a mutable ref
    pub fn get_section_mut(&mut self, idx: u32) -> Option<&mut SourceMapSection> {
        self.sections.get_mut(idx as usize)
    }

    /// Iterates over all sections
    pub fn sections<'a>(&'a self) -> SourceMapSectionIter<'a> {
        SourceMapSectionIter {
            i: self,
            next_idx: 0
        }
    }

    /// Looks up the closest token to a given line and column.
    ///
    /// This requires that the referenced sourcemaps are actually loaded.
    /// If a sourcemap is encountered that is not embedded but just
    /// externally referenced it is silently skipped.
    pub fn lookup_token<'a>(&'a self, line: u32, col: u32) -> Option<Token<'a>> {
        for section in self.sections() {
            let (off_line, off_col) = section.get_offset();
            if off_line < line || off_col < col {
                continue;
            }
            if let Some(map) = section.get_sourcemap() {
                if let Some(tok) = map.lookup_token(line - off_line, col - off_col) {
                    return Some(tok);
                }
            }
        }
        None
    }
}

impl SourceMapSection {

    /// Create a new sourcemap index section
    ///
    /// - `offset`: offset as line and column
    /// - `url`: optional URL of where the sourcemap is located
    /// - `map`: an optional already resolved internal sourcemap
    pub fn new(offset: (u32, u32), url: Option<String>, map: Option<SourceMap>) -> SourceMapSection {
        SourceMapSection {
            offset: offset,
            url: url,
            map: map.map(|x| Box::new(x)),
        }
    }

    /// Returns the offset line
    pub fn get_offset_line(&self) -> u32 {
        self.offset.0
    }

    /// Returns the offset column
    pub fn get_offset_col(&self) -> u32 {
        self.offset.1
    }

    /// Returns the offset as tuple
    pub fn get_offset(&self) -> (u32, u32) {
        self.offset
    }

    /// Returns the URL of the referenced map if available
    pub fn get_url(&self) -> Option<&str> {
        self.url.as_ref().map(|x| &**x)
    }

    /// Returns a reference to the embedded sourcemap if available
    pub fn get_sourcemap(&self) -> Option<&SourceMap> {
        self.map.as_ref().map(|x| &**x)
    }

    /// Replaces the embedded sourcemap
    pub fn set_sourcemap(&mut self, sm: SourceMap) {
        self.map = Some(Box::new(sm));
    }
}
