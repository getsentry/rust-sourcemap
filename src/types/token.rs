use std::{cmp::Ordering, fmt};

use crate::{SourceMap, SourceView};

/// Represents a raw token
///
/// Raw tokens are used internally to represent the sourcemap
/// in a memory efficient way.  If you construct sourcemaps yourself
/// then you need to create these objects, otherwise they are invisible
/// to you as a user.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct RawToken {
    /// the destination (minified) line number (0-indexed)
    pub dst_line: u32,
    /// the destination (minified) column number (0-indexed)
    pub dst_col: u32,
    /// the source line number (0-indexed)
    pub src_line: u32,
    /// the source line column (0-indexed)
    pub src_col: u32,
    /// source identifier
    pub src_id: u32,
    /// name identifier (`!0` in case there is no associated name)
    pub name_id: u32,

    /// If true, this token is a range token.
    ///
    /// See <https://github.com/tc39/source-map-rfc/blob/main/proposals/range-mappings.md>
    pub is_range: bool,
}

/// Represents a token from a sourcemap
#[derive(Copy, Clone)]
pub struct Token<'a> {
    pub(crate) raw: &'a RawToken,
    pub(crate) sm: &'a SourceMap,
    pub(crate) idx: usize,
    pub(crate) offset: u32,
}

impl<'a> Token<'a> {
    /// The sourcemap this token is linked to.
    pub fn sourcemap(&self) -> &'a SourceMap {
        self.sm
    }

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

    /// Get the source line number.
    ///
    /// `u32::MAX` is a sentinel value meaning
    /// this token is unmapped.
    pub fn get_src_line(&self) -> u32 {
        self.raw.src_line
    }

    /// Get the source column number.
    ///
    /// `u32::MAX` is a sentinel value meaning
    /// this token is unmapped.
    pub fn get_src_col(&self) -> u32 {
        self.raw.src_col.saturating_add(self.offset)
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
            self.sm.get_source(self.raw.src_id)
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
            self.sm.get_name(self.raw.name_id)
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

    /// Returns the referenced source view.
    pub fn get_source_view(&self) -> Option<&SourceView> {
        self.sm.get_source_view(self.get_src_id())
    }

    /// If true, this token is a range token.
    ///
    /// See <https://github.com/tc39/source-map-rfc/blob/main/proposals/range-mappings.md>
    pub fn is_range(&self) -> bool {
        self.raw.is_range
    }
}

impl PartialEq for Token<'_> {
    fn eq(&self, other: &Token<'_>) -> bool {
        self.raw == other.raw
    }
}

impl Eq for Token<'_> {}

impl PartialOrd for Token<'_> {
    fn partial_cmp(&self, other: &Token<'_>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Token<'_> {
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
        try_cmp!(self.is_range(), other.is_range());

        Ordering::Equal
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Token {self:#}>")
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}{}",
            self.get_source().unwrap_or("<unknown>"),
            self.get_src_line(),
            self.get_src_col(),
            self.get_name()
                .map(|x| format!(" name={x}"))
                .unwrap_or_default()
        )?;
        if f.alternate() {
            write!(
                f,
                " ({}:{}){}",
                self.get_dst_line(),
                self.get_dst_col(),
                if self.is_range() { " (range)" } else { "" }
            )?;
        }
        Ok(())
    }
}
