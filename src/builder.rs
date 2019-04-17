use std::collections::HashMap;
use std::convert::AsRef;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use url::Url;

use crate::errors::Result;
use crate::types::{RawToken, SourceMap, Token};

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

fn resolve_local_reference(base: &Url, reference: &str) -> Option<PathBuf> {
    let url = match base.join(reference) {
        Ok(url) => {
            if url.scheme() != "file" {
                return None;
            }
            url
        }
        Err(_) => {
            return None;
        }
    };

    url.to_file_path().ok()
}

impl SourceMapBuilder {
    /// Creates a new source map builder and sets the file.
    pub fn new(file: Option<&str>) -> SourceMapBuilder {
        SourceMapBuilder {
            file: file.map(str::to_owned),
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
        self.file = value.map(str::to_owned);
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
        assert!(src_id != !0, "Cannot set sources for tombstone source id");
        self.sources[src_id as usize] = src.to_string();
    }

    /// Looks up a source name for an ID.
    pub fn get_source(&self, src_id: u32) -> Option<&str> {
        self.sources.get(src_id as usize).map(|x| &x[..])
    }

    /// Sets the source contents for an already existing source.
    pub fn set_source_contents(&mut self, src_id: u32, contents: Option<&str>) {
        assert!(src_id != !0, "Cannot set sources for tombstone source id");
        if self.sources.len() > self.source_contents.len() {
            self.source_contents.resize(self.sources.len(), None);
        }
        self.source_contents[src_id as usize] = contents.map(str::to_owned);
    }

    /// Returns the current source contents for a source.
    pub fn get_source_contents(&self, src_id: u32) -> Option<&str> {
        self.source_contents
            .get(src_id as usize)
            .and_then(|x| x.as_ref().map(|x| &x[..]))
    }

    /// Checks if a given source ID has source contents available.
    pub fn has_source_contents(&self, src_id: u32) -> bool {
        self.get_source_contents(src_id).is_some()
    }

    /// Loads source contents from locally accessible files if referenced
    /// accordingly.  Returns the number of loaded source contents
    pub fn load_local_source_contents(&mut self, base_path: Option<&Path>) -> Result<usize> {
        let mut abs_path = env::current_dir()?;
        if let Some(path) = base_path {
            abs_path.push(path);
        }
        let base_url = Url::from_directory_path(&abs_path).unwrap();

        let mut to_read = vec![];
        for (source, &src_id) in self.source_map.iter() {
            if self.has_source_contents(src_id) {
                continue;
            }
            if let Some(path) = resolve_local_reference(&base_url, &source) {
                to_read.push((src_id, path));
            }
        }

        let rv = to_read.len();
        for (src_id, path) in to_read {
            if let Ok(mut f) = fs::File::open(&path) {
                let mut contents = String::new();
                if f.read_to_string(&mut contents).is_ok() {
                    self.set_source_contents(src_id, Some(&contents));
                }
            }
        }

        Ok(rv)
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
    pub fn add(
        &mut self,
        dst_line: u32,
        dst_col: u32,
        src_line: u32,
        src_col: u32,
        source: Option<&str>,
        name: Option<&str>,
    ) -> RawToken {
        let src_id = match source {
            Some(source) => self.add_source(source),
            None => !0,
        };
        let name_id = match name {
            Some(name) => self.add_name(name),
            None => !0,
        };
        let raw = RawToken {
            dst_line,
            dst_col,
            src_line,
            src_col,
            src_id,
            name_id,
        };
        self.tokens.push(raw);
        raw
    }

    /// Shortcut for adding a new mapping based of an already existing token,
    /// optionally removing the name.
    pub fn add_token(&mut self, token: &Token<'_>, with_name: bool) -> RawToken {
        let name = if with_name { token.get_name() } else { None };
        self.add(
            token.get_dst_line(),
            token.get_dst_col(),
            token.get_src_line(),
            token.get_src_col(),
            token.get_source(),
            name,
        )
    }

    /// Strips common prefixes from the sources in the builder
    pub fn strip_prefixes<S: AsRef<str>>(&mut self, prefixes: &[S]) {
        for source in self.sources.iter_mut() {
            for prefix in prefixes {
                let mut prefix = prefix.as_ref().to_string();
                if !prefix.ends_with('/') {
                    prefix.push('/');
                }
                if source.starts_with(&prefix) {
                    *source = source[prefix.len()..].to_string();
                    break;
                }
            }
        }
    }

    /// Converts the builder into a sourcemap.
    pub fn into_sourcemap(self) -> SourceMap {
        let contents = if !self.source_contents.is_empty() {
            Some(self.source_contents)
        } else {
            None
        };
        SourceMap::new(self.file, self.tokens, self.names, self.sources, contents)
    }
}
