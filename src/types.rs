use std::cmp::Ordering;
use std::fmt;
use std::io::{Read, Write};
use std::path::Path;

use crate::builder::SourceMapBuilder;
use crate::decoder::{decode, decode_slice};
use crate::encoder::encode;
use crate::errors::{Error, Result};
use crate::sourceview::SourceView;
use crate::utils::find_common_prefix;

/// Controls the `SourceMap::rewrite` behavior
///
/// Default configuration:
///
/// * `with_names`: true
/// * `with_source_contents`: true
/// * `load_local_source_contents`: false
pub struct RewriteOptions<'a> {
    /// If enabled, names are kept in the rewritten sourcemap.
    pub with_names: bool,
    /// If enabled source contents are kept in the sourcemap.
    pub with_source_contents: bool,
    /// If enabled local source contents that are not in the
    /// file are automatically inlined.
    pub load_local_source_contents: bool,
    /// The base path to the used for source reference resolving
    /// when loading local source contents is used.
    pub base_path: Option<&'a Path>,
    /// Optionally strips common prefixes from the sources.  If
    /// an item in the list is set to `~` then the common prefix
    /// of all sources is stripped.
    pub strip_prefixes: &'a [&'a str],
}

impl<'a> Default for RewriteOptions<'a> {
    fn default() -> RewriteOptions<'a> {
        RewriteOptions {
            with_names: true,
            with_source_contents: true,
            load_local_source_contents: false,
            base_path: None,
            strip_prefixes: &[][..],
        }
    }
}

/// Represents the result of a decode operation
///
/// This represents either an actual sourcemap or a source map index.
/// Usually the two things are too distinct to provide a common
/// interface however for token lookup and writing back into a writer
/// general methods are provided.
pub enum DecodedMap {
    /// Indicates a regular sourcemap
    Regular(SourceMap),
    /// Indicates a sourcemap index
    Index(SourceMapIndex),
}

impl DecodedMap {
    /// Alias for `decode`.
    pub fn from_reader<R: Read>(rdr: R) -> Result<DecodedMap> {
        decode(rdr)
    }

    /// Writes a decoded sourcemap to a writer.
    pub fn to_writer<W: Write>(&self, w: W) -> Result<()> {
        match *self {
            DecodedMap::Regular(ref sm) => encode(sm, w),
            DecodedMap::Index(ref smi) => encode(smi, w),
        }
    }

    /// Shortcut to look up a token on either an index or a
    /// regular sourcemap.  This method can only be used if
    /// the contained index actually contains embedded maps
    /// or it will not be able to look up anything.
    pub fn lookup_token(&self, line: u32, col: u32) -> Option<Token<'_>> {
        match *self {
            DecodedMap::Regular(ref sm) => sm.lookup_token(line, col),
            DecodedMap::Index(ref smi) => smi.lookup_token(line, col),
        }
    }
}

/// Represents a raw token
///
/// Raw tokens are used internally to represent the sourcemap
/// in a memory efficient way.  If you construct sourcemaps yourself
/// then you need to create these objects, otherwise they are invisible
/// to you as a user.
#[derive(PartialEq, Copy, Clone, Debug)]
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
#[derive(Copy, Clone)]
pub struct Token<'a> {
    raw: &'a RawToken,
    i: &'a SourceMap,
    idx: u32,
}

impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Token<'_>) -> bool {
        self.raw == other.raw
    }
}

impl<'a> Eq for Token<'a> {}

impl<'a> PartialOrd for Token<'a> {
    fn partial_cmp(&self, other: &Token<'_>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Token<'a> {
    fn cmp(&self, other: &Token<'_>) -> Ordering {
        macro_rules! try_cmp {
            ($a:expr, $b:expr) => {
                match $a.cmp(&$b) {
                    Ordering::Equal => {}
                    x => {
                        return x;
                    }
                }
            };
        }
        try_cmp!(self.get_dst_line(), other.get_dst_line());
        try_cmp!(self.get_dst_col(), other.get_dst_col());
        try_cmp!(self.get_source(), other.get_source());
        try_cmp!(self.get_src_line(), other.get_src_line());
        try_cmp!(self.get_src_col(), other.get_src_col());
        try_cmp!(self.get_name(), other.get_name());
        Ordering::Equal
    }
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

    /// Return the source ID of the token
    pub fn get_src_id(&self) -> u32 {
        self.raw.src_id
    }

    /// get the source if it exists as string
    pub fn get_source(&self) -> Option<&'a str> {
        if self.raw.src_id == !0 {
            None
        } else {
            self.i.get_source(self.raw.src_id)
        }
    }

    /// Is there a source for this token?
    pub fn has_source(&self) -> bool {
        self.raw.src_id != !0
    }

    /// get the name if it exists as string
    pub fn get_name(&self) -> Option<&'a str> {
        if self.raw.name_id == !0 {
            None
        } else {
            self.i.get_name(self.raw.name_id)
        }
    }

    /// returns `true` if a name exists, `false` otherwise
    pub fn has_name(&self) -> bool {
        self.get_name().is_some()
    }

    /// Return the name ID of the token
    pub fn get_name_id(&self) -> u32 {
        self.raw.name_id
    }

    /// Converts the token into a debug tuple in the form
    /// `(source, src_line, src_col, name)`
    pub fn to_tuple(&self) -> (&'a str, u32, u32, Option<&'a str>) {
        (
            self.get_source().unwrap_or(""),
            self.get_src_line(),
            self.get_src_col(),
            self.get_name(),
        )
    }

    /// Get the underlying raw token
    pub fn get_raw_token(&self) -> RawToken {
        *self.raw
    }
}

pub fn idx_from_token(token: &Token<'_>) -> u32 {
    token.idx
}

pub fn sourcemap_from_token<'a>(token: &Token<'a>) -> &'a SourceMap {
    token.i
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

/// Iterates over all sources in a sourcemap
pub struct SourceIter<'a> {
    i: &'a SourceMap,
    next_idx: u32,
}

impl<'a> Iterator for SourceIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.i.get_source(self.next_idx).map(|source| {
            self.next_idx += 1;
            source
        })
    }
}

/// Iterates over all source contents in a sourcemap
pub struct SourceContentsIter<'a> {
    i: &'a SourceMap,
    next_idx: u32,
}

impl<'a> Iterator for SourceContentsIter<'a> {
    type Item = Option<&'a str>;

    fn next(&mut self) -> Option<Option<&'a str>> {
        if self.next_idx >= self.i.get_source_count() {
            None
        } else {
            let rv = Some(self.i.get_source_contents(self.next_idx));
            self.next_idx += 1;
            rv
        }
    }
}

/// Iterates over all tokens in a sourcemap
pub struct NameIter<'a> {
    i: &'a SourceMap,
    next_idx: u32,
}

impl<'a> Iterator for NameIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.i.get_name(self.next_idx).map(|name| {
            self.next_idx += 1;
            name
        })
    }
}

/// Iterates over all index items in a sourcemap
pub struct IndexIter<'a> {
    i: &'a SourceMap,
    next_idx: usize,
}

impl<'a> Iterator for IndexIter<'a> {
    type Item = (u32, u32, u32);

    fn next(&mut self) -> Option<(u32, u32, u32)> {
        self.i.index.get(self.next_idx).map(|idx| {
            self.next_idx += 1;
            *idx
        })
    }
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Token {:#}>", self)
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}{}",
            self.get_source().unwrap_or("<unknown>"),
            self.get_src_line(),
            self.get_src_col(),
            self.get_name()
                .map(|x| format!(" name={}", x))
                .unwrap_or_default()
        )?;
        if f.alternate() {
            write!(f, " ({}:{})", self.get_dst_line(), self.get_dst_col())?;
        }
        Ok(())
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
    file: Option<String>,
    sections: Vec<SourceMapSection>,
}

/// Represents a sourcemap in memory
///
/// This is always represents a regular "non-indexed" sourcemap.  Particularly
/// in case the `from_reader` method is used an index sourcemap will be
/// rejected with an error on reading.
pub struct SourceMap {
    file: Option<String>,
    tokens: Vec<RawToken>,
    index: Vec<(u32, u32, u32)>,
    names: Vec<String>,
    sources: Vec<String>,
    sources_content: Vec<Option<SourceView<'static>>>,
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
    ///
    /// While sourcemaps objects permit some modifications, it's generally
    /// not possible to modify tokens after they have been added.  For
    /// creating sourcemaps from scratch or for general operations for
    /// modifying a sourcemap have a look at the `SourceMapBuilder`.
    pub fn from_reader<R: Read>(rdr: R) -> Result<SourceMap> {
        match decode(rdr)? {
            DecodedMap::Regular(sm) => Ok(sm),
            DecodedMap::Index(_) => Err(Error::IndexedSourcemap),
        }
    }

    /// Writes a sourcemap into a writer.
    ///
    /// Note that this operation will generate an equivalent sourcemap to the
    /// one that was generated on load however there might be small differences
    /// in the generated JSON and layout.  For instance `sourceRoot` will not
    /// be set as upon parsing of the sourcemap the sources will already be
    /// expanded.
    ///
    /// ```rust
    /// # use sourcemap::SourceMap;
    /// # let input: &[_] = b"{
    /// #     \"version\":3,
    /// #     \"sources\":[\"coolstuff.js\"],
    /// #     \"names\":[\"x\",\"alert\"],
    /// #     \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
    /// # }";
    /// let sm = SourceMap::from_reader(input).unwrap();
    /// let mut output : Vec<u8> = vec![];
    /// sm.to_writer(&mut output).unwrap();
    /// ```
    pub fn to_writer<W: Write>(&self, w: W) -> Result<()> {
        encode(self, w)
    }

    /// Creates a sourcemap from a reader over a JSON byte slice in UTF-8
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
    /// let sm = SourceMap::from_slice(input).unwrap();
    /// ```
    pub fn from_slice(slice: &[u8]) -> Result<SourceMap> {
        match decode_slice(slice)? {
            DecodedMap::Regular(sm) => Ok(sm),
            DecodedMap::Index(_) => Err(Error::IndexedSourcemap),
        }
    }

    /// Constructs a new sourcemap from raw components.
    ///
    /// - `file`: an optional filename of the sourcemap
    /// - `tokens`: a list of raw tokens
    /// - `names`: a vector of names
    /// - `sources` a vector of source filenames
    /// - `sources_content` optional source contents
    pub fn new(
        file: Option<String>,
        tokens: Vec<RawToken>,
        names: Vec<String>,
        sources: Vec<String>,
        sources_content: Option<Vec<Option<String>>>,
    ) -> SourceMap {
        let mut index: Vec<_> = tokens
            .iter()
            .enumerate()
            .map(|(idx, token)| (token.dst_line, token.dst_col, idx as u32))
            .collect();
        index.sort();
        SourceMap {
            file,
            tokens,
            index,
            names,
            sources,
            sources_content: sources_content
                .unwrap_or_default()
                .into_iter()
                .map(|opt| opt.map(SourceView::from_string))
                .collect(),
        }
    }

    /// Returns the embedded filename in case there is one.
    pub fn get_file(&self) -> Option<&str> {
        self.file.as_ref().map(|x| &x[..])
    }

    /// Sets a new value for the file.
    pub fn set_file(&mut self, value: Option<&str>) {
        self.file = value.map(str::to_owned);
    }

    /// Looks up a token by its index.
    pub fn get_token(&self, idx: u32) -> Option<Token<'_>> {
        self.tokens
            .get(idx as usize)
            .map(|raw| Token { raw, i: self, idx })
    }

    /// Returns the number of tokens in the sourcemap.
    pub fn get_token_count(&self) -> u32 {
        self.tokens.len() as u32
    }

    /// Returns an iterator over the tokens.
    pub fn tokens(&self) -> TokenIter<'_> {
        TokenIter {
            i: self,
            next_idx: 0,
        }
    }

    /// Looks up the closest token to a given line and column.
    pub fn lookup_token(&self, line: u32, col: u32) -> Option<Token<'_>> {
        let mut low = 0;
        let mut high = self.index.len();

        while low < high {
            let mid = (low + high) / 2;
            let ii = &self.index[mid as usize];
            if (line, col) < (ii.0, ii.1) {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        if low > 0 && low <= self.index.len() {
            self.get_token(self.index[low as usize - 1].2)
        } else {
            None
        }
    }

    /// Given a location, name and minified source file resolve a minified
    /// name to an original function name.
    ///
    /// This invokes some guesswork and requires access to the original minified
    /// source.  This will not yield proper results for anonymous functions or
    /// functions that do not have clear function names.  (For instance it's
    /// recommended that dotted function names are not passed to this
    /// function).
    pub fn get_original_function_name<'a>(
        &self,
        line: u32,
        col: u32,
        minified_name: &str,
        sv: &'a SourceView<'a>,
    ) -> Option<&str> {
        self.lookup_token(line, col)
            .and_then(|token| sv.get_original_function_name(token, minified_name))
    }

    /// Returns the number of sources in the sourcemap.
    pub fn get_source_count(&self) -> u32 {
        self.sources.len() as u32
    }

    /// Looks up a source for a specific index.
    pub fn get_source(&self, idx: u32) -> Option<&str> {
        self.sources.get(idx as usize).map(|x| &x[..])
    }

    /// Sets a new source value for an index.  This cannot add new
    /// sources.
    ///
    /// This panics if a source is set that does not exist.
    pub fn set_source(&mut self, idx: u32, value: &str) {
        self.sources[idx as usize] = value.to_string();
    }

    /// Iterates over all sources
    pub fn sources(&self) -> SourceIter<'_> {
        SourceIter {
            i: self,
            next_idx: 0,
        }
    }

    /// Returns the sources content as source view.
    pub fn get_source_view(&self, idx: u32) -> Option<&SourceView<'_>> {
        self.sources_content
            .get(idx as usize)
            .and_then(Option::as_ref)
    }

    /// Looks up the content for a source.
    pub fn get_source_contents(&self, idx: u32) -> Option<&str> {
        self.sources_content
            .get(idx as usize)
            .and_then(Option::as_ref)
            .map(SourceView::source)
    }

    /// Sets source contents for a source.
    pub fn set_source_contents(&mut self, idx: u32, value: Option<&str>) {
        if self.sources_content.len() != self.sources.len() {
            self.sources_content.resize(self.sources.len(), None);
        }
        self.sources_content[idx as usize] = value.map(|x| SourceView::from_string(x.to_string()));
    }

    /// Iterates over all source contents
    pub fn source_contents(&self) -> SourceContentsIter<'_> {
        SourceContentsIter {
            i: self,
            next_idx: 0,
        }
    }

    /// Returns an iterator over the names.
    pub fn names(&self) -> NameIter<'_> {
        NameIter {
            i: self,
            next_idx: 0,
        }
    }

    /// Returns the number of names in the sourcemap.
    pub fn get_name_count(&self) -> u32 {
        self.names.len() as u32
    }

    /// Returns true if there are any names in the map.
    pub fn has_names(&self) -> bool {
        !self.names.is_empty()
    }

    /// Looks up a name for a specific index.
    pub fn get_name(&self, idx: u32) -> Option<&str> {
        self.names.get(idx as usize).map(|x| &x[..])
    }

    /// Removes all names from the sourcemap.
    pub fn remove_names(&mut self) {
        self.names.clear();
    }

    /// Returns the number of items in the index
    pub fn get_index_size(&self) -> usize {
        self.index.len()
    }

    /// Returns the number of items in the index
    pub fn index_iter(&self) -> IndexIter<'_> {
        IndexIter {
            i: self,
            next_idx: 0,
        }
    }

    /// This rewrites the sourcemap accoridng to the provided rewrite
    /// options.
    ///
    /// The default behavior is to just deduplicate the sourcemap, something
    /// that automatically takes place.  This for instance can be used to
    /// slightly compress sourcemaps if certain data is not wanted.
    ///
    /// ```rust
    /// use sourcemap::{SourceMap, RewriteOptions};
    /// # let input: &[_] = b"{
    /// #     \"version\":3,
    /// #     \"sources\":[\"coolstuff.js\"],
    /// #     \"names\":[\"x\",\"alert\"],
    /// #     \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
    /// # }";
    /// let sm = SourceMap::from_slice(input).unwrap();
    /// let new_sm = sm.rewrite(&RewriteOptions {
    ///     with_names: false,
    ///     ..Default::default()
    /// });
    /// ```
    pub fn rewrite(self, options: &RewriteOptions<'_>) -> Result<SourceMap> {
        let mut builder = SourceMapBuilder::new(self.get_file());

        for token in self.tokens() {
            let raw = builder.add_token(&token, options.with_names);
            if raw.src_id != !0
                && options.with_source_contents
                && !builder.has_source_contents(raw.src_id)
            {
                builder
                    .set_source_contents(raw.src_id, self.get_source_contents(token.get_src_id()));
            }
        }
        if options.load_local_source_contents {
            builder.load_local_source_contents(options.base_path)?;
        }

        let mut prefixes = vec![];
        let mut need_common_prefix = false;
        for &prefix in options.strip_prefixes.iter() {
            if prefix == "~" {
                need_common_prefix = true;
            } else {
                prefixes.push(prefix.to_string());
            }
        }
        if need_common_prefix {
            if let Some(prefix) = find_common_prefix(self.sources.iter().map(String::as_str)) {
                prefixes.push(prefix);
            }
        }
        if !prefixes.is_empty() {
            builder.strip_prefixes(&prefixes);
        }

        Ok(builder.into_sourcemap())
    }
}

impl SourceMapIndex {
    /// Creates a sourcemap index from a reader over a JSON stream in UTF-8
    /// format.  Optionally a "garbage header" as defined by the
    /// sourcemap draft specification is supported.  In case a regular
    /// sourcemap is encountered an error is returned.
    pub fn from_reader<R: Read>(rdr: R) -> Result<SourceMapIndex> {
        match decode(rdr)? {
            DecodedMap::Regular(_) => Err(Error::RegularSourcemap),
            DecodedMap::Index(smi) => Ok(smi),
        }
    }

    /// Writes a sourcemap index into a writer.
    pub fn to_writer<W: Write>(&self, w: W) -> Result<()> {
        encode(self, w)
    }

    /// Creates a sourcemap index from a reader over a JSON byte slice in UTF-8
    /// format.  Optionally a "garbage header" as defined by the
    /// sourcemap draft specification is supported.  In case a regular
    /// sourcemap is encountered an error is returned.
    pub fn from_slice(slice: &[u8]) -> Result<SourceMapIndex> {
        match decode_slice(slice)? {
            DecodedMap::Regular(_) => Err(Error::RegularSourcemap),
            DecodedMap::Index(smi) => Ok(smi),
        }
    }

    /// Constructs a new sourcemap index from raw components.
    ///
    /// - `file`: an optional filename of the index
    /// - `sections`: a vector of source map index sections
    pub fn new(file: Option<String>, sections: Vec<SourceMapSection>) -> SourceMapIndex {
        SourceMapIndex { file, sections }
    }

    /// Returns the embedded filename in case there is one.
    pub fn get_file(&self) -> Option<&str> {
        self.file.as_ref().map(|x| &x[..])
    }

    /// Sets a new value for the file.
    pub fn set_file(&mut self, value: Option<&str>) {
        self.file = value.map(str::to_owned);
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
    pub fn sections(&self) -> SourceMapSectionIter<'_> {
        SourceMapSectionIter {
            i: self,
            next_idx: 0,
        }
    }

    /// Looks up the closest token to a given line and column.
    ///
    /// This requires that the referenced sourcemaps are actually loaded.
    /// If a sourcemap is encountered that is not embedded but just
    /// externally referenced it is silently skipped.
    pub fn lookup_token(&self, line: u32, col: u32) -> Option<Token<'_>> {
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

    /// Flattens an indexed sourcemap into a regular one.  This requires
    /// that all referenced sourcemaps are attached.
    pub fn flatten(self) -> Result<SourceMap> {
        let mut builder = SourceMapBuilder::new(self.get_file());

        for section in self.sections() {
            let (off_line, off_col) = section.get_offset();
            let map = match section.get_sourcemap() {
                Some(map) => map,
                None => {
                    return Err(Error::CannotFlatten(format!(
                        "Section has an unresolved \
                         sourcemap: {}",
                        section.get_url().unwrap_or("<unknown url>")
                    )));
                }
            };

            for token in map.tokens() {
                let raw = builder.add(
                    token.get_dst_line() + off_line,
                    token.get_dst_col() + off_col,
                    token.get_src_line(),
                    token.get_src_col(),
                    token.get_source(),
                    token.get_name(),
                );
                if token.get_source().is_some() && !builder.has_source_contents(raw.src_id) {
                    builder.set_source_contents(
                        raw.src_id,
                        map.get_source_contents(token.get_src_id()),
                    );
                }
            }
        }

        Ok(builder.into_sourcemap())
    }

    /// Flattens an indexed sourcemap into a regular one and automatically
    /// rewrites it.  This is more useful than plain flattening as this will
    /// cause the sourcemap to be properly deduplicated.
    pub fn flatten_and_rewrite(self, options: &RewriteOptions<'_>) -> Result<SourceMap> {
        self.flatten()?.rewrite(options)
    }
}

impl SourceMapSection {
    /// Create a new sourcemap index section
    ///
    /// - `offset`: offset as line and column
    /// - `url`: optional URL of where the sourcemap is located
    /// - `map`: an optional already resolved internal sourcemap
    pub fn new(
        offset: (u32, u32),
        url: Option<String>,
        map: Option<SourceMap>,
    ) -> SourceMapSection {
        SourceMapSection {
            offset,
            url,
            map: map.map(Box::new),
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

    /// Updates the URL for this section.
    pub fn set_url(&mut self, value: Option<&str>) {
        self.url = value.map(str::to_owned);
    }

    /// Returns a reference to the embedded sourcemap if available
    pub fn get_sourcemap(&self) -> Option<&SourceMap> {
        self.map.as_ref().map(Box::as_ref)
    }

    /// Returns a reference to the embedded sourcemap if available
    pub fn get_sourcemap_mut(&mut self) -> Option<&mut SourceMap> {
        self.map.as_mut().map(Box::as_mut)
    }

    /// Replaces the embedded sourcemap
    pub fn set_sourcemap(&mut self, sm: Option<SourceMap>) {
        self.map = sm.map(Box::new);
    }
}
