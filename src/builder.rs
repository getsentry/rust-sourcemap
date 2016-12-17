use std::collections::HashMap;

use types::{SourceMap, RawToken, Token};


/// Helper for sourcemap generation
///
/// This helper exists because generating and modifying `SourceMap`
/// objects is generally not very comfortable.  As a general aid this
/// type can help.
pub struct SourceMapBuilder {
    file: Option<String>,
    name_map: HashMap<String, u32>,
    names: Vec<String>,
    tokens: Vec<RawToken>,
    source_map: HashMap<String, u32>,
    sources: Vec<String>,
    source_contents: Vec<Option<String>>,
}

impl SourceMapBuilder {

    /// Creates a new source map builder and sets the file.
    pub fn new(file: Option<&str>) -> SourceMapBuilder {
        SourceMapBuilder {
            file: file.map(|x| x.to_string()),
            name_map: HashMap::new(),
            names: vec![],
            tokens: vec![],
            source_map: HashMap::new(),
            sources: vec![],
            source_contents: vec![],
        }
    }

    /// Sets the file for the sourcemap (optional)
    pub fn set_file(&mut self, value: Option<&str>) {
        self.file = value.map(|x| x.to_string());
    }

    /// Returns the currently set file.
    pub fn get_file(&self) -> Option<&str> {
        self.file.as_ref().map(|x| &x[..])
    }

    /// Registers a new source with the builder and returns the source ID.
    pub fn add_source(&mut self, src: &str) -> u32 {
        let count = self.sources.len() as u32;
        let id = *self.source_map.entry(src.into()).or_insert(count);
        if id == count {
            self.sources.push(src.into());
        }
        id
    }

    /// Changes the source name for an already set source.
    pub fn set_source(&mut self, src_id: u32, src: &str) {
        self.sources[src_id as usize] = src.to_string();
    }

    /// Looks up a source name for an ID.
    pub fn get_source(&self, src_id: u32) -> Option<&str> {
        self.sources.get(src_id as usize).map(|x| &x[..])
    }

    /// Sets the source contents for an already existing source.
    pub fn set_source_contents(&mut self, src_id: u32, contents: Option<&str>) {
        if self.sources.len() > self.source_contents.len() {
            self.source_contents.resize(self.sources.len(), None);
        }
        self.source_contents[src_id as usize] = contents.map(|x| x.to_string());
    }

    /// Returns the current source contents for a source.
    pub fn get_source_contents(&self, src_id: u32) -> Option<&str> {
        self.source_contents.get(src_id as usize).and_then(|x| x.as_ref().map(|x| &x[..]))
    }

    /// Checks if a given source ID has source contents available.
    pub fn has_source_contents(&self, src_id: u32) -> bool {
        self.get_source_contents(src_id).is_some()
    }

    /// Registers a name with the builder and returns the name ID.
    pub fn add_name(&mut self, name: &str) -> u32 {
        let count = self.names.len() as u32;
        let id = *self.name_map.entry(name.into()).or_insert(count);
        if id == count {
            self.names.push(name.into());
        }
        id
    }

    /// Adds a new mapping to the builder.
    pub fn add(&mut self, dst_line: u32, dst_col: u32, src_line: u32,
               src_col: u32, source: Option<&str>,
               name: Option<&str>) -> RawToken {
        let src_id = match source {
            Some(source) => self.add_source(source),
            None => !0
        };
        let name_id = match name {
            Some(name) => self.add_name(name),
            None => !0
        };
        let raw = RawToken {
            dst_line: dst_line,
            dst_col: dst_col,
            src_line: src_line,
            src_col: src_col,
            src_id: src_id,
            name_id: name_id,
        };
        self.tokens.push(raw);
        raw
    }

    /// Shortcut for adding a new mapping based of an already existing token,
    /// optionally removing the name.
    pub fn add_token(&mut self, token: &Token, with_name: bool) -> RawToken {
        let name = if with_name { token.get_name() } else { None };
        self.add(token.get_dst_line(), token.get_dst_col(),
                 token.get_src_line(), token.get_src_col(),
                 token.get_source(), name)
    }

    /// Converts the builder into a sourcemap.
    pub fn into_sourcemap(self) -> SourceMap {
        let contents = if self.source_contents.len() > 0 {
            Some(self.source_contents)
        } else {
            None
        };
        SourceMap::new(self.file, self.tokens, self.names,
                       self.sources, contents)
    }
}
