use std::collections::HashMap;

use types::{SourceMap, RawToken, Token};


/// Helper for sourcemap generation
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
    pub fn new() -> SourceMapBuilder {
        SourceMapBuilder {
            file: None,
            name_map: HashMap::new(),
            names: vec![],
            tokens: vec![],
            source_map: HashMap::new(),
            sources: vec![],
            source_contents: vec![],
        }
    }

    pub fn set_file(&mut self, value: Option<&str>) {
        self.file = value.map(|x| x.to_string());
    }

    pub fn get_file(&self) -> Option<&str> {
        self.file.as_ref().map(|x| &x[..])
    }

    pub fn add_source(&mut self, src: &str) -> u32 {
        let count = self.sources.len() as u32;
        let id = *self.source_map.entry(src.into()).or_insert(count);
        if id == count {
            self.sources.push(src.into());
        }
        id
    }

    pub fn set_source(&mut self, src_id: u32, src: &str) {
        self.sources[src_id as usize] = src.to_string();
    }

    pub fn get_source(&self, src_id: u32) -> Option<&str> {
        self.sources.get(src_id as usize).map(|x| &x[..])
    }

    pub fn set_source_contents(&mut self, src_id: u32, contents: Option<&str>) {
        if self.sources.len() > self.source_contents.len() {
            self.source_contents.resize(self.sources.len(), None);
        }
        self.source_contents[src_id as usize] = contents.map(|x| x.to_string());
    }

    pub fn get_source_contents(&self, src_id: u32) -> Option<&str> {
        self.source_contents.get(src_id as usize).and_then(|x| x.as_ref().map(|x| &x[..]))
    }

    pub fn has_source_contents(&self, src_id: u32) -> bool {
        self.get_source_contents(src_id).is_some()
    }

    pub fn add_name(&mut self, name: &str) -> u32 {
        let count = self.names.len() as u32;
        let id = *self.name_map.entry(name.into()).or_insert(count);
        if id == count {
            self.names.push(name.into());
        }
        id
    }

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

    pub fn add_token(&mut self, token: &Token, with_name: bool) -> RawToken {
        let name = if with_name { token.get_name() } else { None };
        self.add(token.get_dst_line(), token.get_dst_col(),
                 token.get_src_line(), token.get_src_col(),
                 token.get_source(), name)
    }

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
