use crate::SourceMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RawOriginalScope {
    pub(crate) start_line: u32,
    pub(crate) start_col: u32,
    pub(crate) end_line: u32,
    pub(crate) end_col: u32,
    pub(crate) name_id: u32,
    pub(crate) kind_id: u32,
    pub(crate) is_stack_frame: bool,
    pub(crate) children: Vec<usize>,
    pub(crate) parent: Option<usize>,
}

/// A lexical scope in an original source file.
#[derive(Debug, Clone)]
pub struct OriginalScope<'a> {
    sm: &'a SourceMap,
    scopes: &'a [RawOriginalScope],
    raw: &'a RawOriginalScope,
}

impl<'a> OriginalScope<'a> {
    /// The original name of this scope (e.g. the class or function name).
    pub fn get_name(&self) -> Option<&'a str> {
        if self.raw.name_id == !0 {
            None
        } else {
            self.sm.get_name(self.raw.name_id)
        }
    }

    /// Get this scope's kind.
    pub fn get_kind(&self) -> Option<&'a str> {
        if self.raw.kind_id == !0 {
            None
        } else {
            self.sm.get_name(self.raw.kind_id)
        }
    }

    /// Get the scope's (inclusive) start position.
    pub fn get_start(&self) -> (u32, u32) {
        (self.raw.start_line, self.raw.start_col)
    }

    /// Get the scope's (exclusive) end position.
    pub fn get_end(&self) -> (u32, u32) {
        (self.raw.end_line, self.raw.end_col)
    }

    /// Whether this scope would introduce a stack frame when run.
    pub fn is_stack_frame(&self) -> bool {
        self.raw.is_stack_frame
    }

    /// Returns an iterator over this scope's children.
    pub fn children(&self) -> OriginalScopeIter<'a> {
        OriginalScopeIter {
            sm: self.sm,
            scopes: self.scopes,
            indices: &self.raw.children[..],
        }
    }

    /// Returns this scope's parent, or `None` if it is a root scope.
    pub fn parent(&self) -> Option<Self> {
        let parent_idx = self.raw.parent?;
        let raw = self.scopes.get(parent_idx)?;

        Some(Self {
            sm: self.sm,
            scopes: self.scopes,
            raw,
        })
    }
}

pub struct OriginalScopeIter<'a> {
    sm: &'a SourceMap,
    scopes: &'a [RawOriginalScope],
    indices: &'a [usize],
}

impl<'a> Iterator for OriginalScopeIter<'a> {
    type Item = OriginalScope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (next, rest) = self.indices.split_first()?;
        self.indices = rest;
        let raw = self.scopes.get(*next)?;
        Some(OriginalScope {
            sm: self.sm,
            scopes: self.scopes,
            raw,
        })
    }
}

impl SourceMap {
    /// Returns an iterator over the root scopes for a source.
    pub fn scopes_for_source(&self, source_id: u32) -> OriginalScopeIter<'_> {
        let (scopes, indices) = match self.scopes.get(&(source_id as usize)) {
            Some(scopes) => (&scopes.all_scopes[..], &scopes.roots[..]),
            None => (&[][..], &[][..]),
        };

        OriginalScopeIter {
            sm: self,
            scopes,
            indices,
        }
    }
}

/// A collection of scopes for one source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OriginalScopes {
    /// A forest of scopes.
    ///
    /// Scopes without a parent are root scopes.
    /// A scope's parent index as well as its children indices
    /// are the indices of its parent and children in this list,
    /// respectively.
    pub(crate) all_scopes: Vec<RawOriginalScope>,
    /// The list of indices of root scopes in this collection.
    pub(crate) roots: Vec<usize>,
}
