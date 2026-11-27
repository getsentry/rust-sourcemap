use crate::types::{OriginalScopes, RawOriginalScope};
use crate::vlq;
use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
struct PositionAccumulator {
    line: u32,
    column: u32,
}

impl PositionAccumulator {
    fn advance(&mut self, line_incr: u32, col_incr: u32) -> (u32, u32) {
        if line_incr == 0 {
            self.column = self.column.saturating_add(col_incr);
        } else {
            self.line = self.line.saturating_add(line_incr);
            self.column = col_incr;
        }

        (self.line, self.column)
    }
}

#[derive(Debug, Clone, Default)]
struct IndexAccumulator(u32);

impl IndexAccumulator {
    fn add(&mut self, incr: i32) -> u32 {
        self.0 = self.0.saturating_add_signed(incr);
        self.0
    }
}

/// Running state for decoding the scopes for a source.
#[derive(Debug, Clone, Default)]
struct DecodeScopeState {
    /// The running position, used for the start and end of scopes.
    position: PositionAccumulator,
    /// The running name index.
    name_idx: IndexAccumulator,
    /// The running kind index.
    kind_idx: IndexAccumulator,
}

/// Scope start entries are of the form
/// `B <flags> <line> <column> <name_or_kind>? <kind>?`.
///
/// This function assumes that the leading `B` has already been stripped, only the
/// numbers remain. `<line>` and `<column>` are relative offsets used to advance
/// the current position, `<name>` and `<kind>` are relative offsets used to advance
/// the respective indices in the decoding state.
///
/// This returns a partial `RawOriginalScope` in which the children and end position
/// aren't filled in yet. They are backfilled when the scope is closed by `decode_scope_end`.
fn decode_scope_start(item: &str, state: &mut DecodeScopeState) -> Result<RawOriginalScope> {
    // Indicates that `<name_or_kind>` is present and contains a name index.
    const NAME_FLAG: u32 = 0x1;
    // Indicates that either `<name_or_kind>` or `<kind>` is present and contains
    // a kind index, depending on whether `NAME_FLAG` is also set.
    const KIND_FLAG: u32 = 0x2;
    // Indicates that this scope creates a stack frame.
    const STACKFRAME_FLAG: u32 = 0x4;

    let mut vlqs = vlq::decode_vlqs(item);

    let flags = vlqs.next_required()?.unsigned();
    let line = vlqs.next_required()?.unsigned();
    let column = vlqs.next_required()?.unsigned();

    let has_name = (flags & NAME_FLAG) > 0;
    let has_kind = (flags & KIND_FLAG) > 0;
    let is_stack_frame = (flags & STACKFRAME_FLAG) > 0;

    let (name_id, kind_id) = match (has_name, has_kind) {
        (false, false) => (None, None),
        (true, false) => (Some(vlqs.next_required()?), None),
        (false, true) => (None, Some(vlqs.next_required()?)),
        (true, true) => (Some(vlqs.next_required()?), Some(vlqs.next_required()?)),
    };

    let (start_line, start_col) = state.position.advance(line, column);
    let name_id = name_id.map_or(u32::MAX, |name_incr| state.name_idx.add(name_incr.signed()));
    let kind_id = kind_id.map_or(u32::MAX, |kind_incr| state.kind_idx.add(kind_incr.signed()));

    Ok(RawOriginalScope {
        start_line,
        start_col,
        end_line: start_line,
        end_col: start_col,
        name_id,
        kind_id,
        is_stack_frame,
        parent: None,
        children: Vec::new(),
    })
}

/// Scope start entries are of the form `C <line> <column>`.
///
/// This function assumes that the leading `C` has already been stripped, only the
/// numbers remain. `<line>` and `<column>` are relative offsets used to advance
/// the current position.
///
/// This returns a `line` and `column` that are used to fill in the end position
/// of the scope closed by this record.
fn decode_scope_end(item: &str, state: &mut DecodeScopeState) -> Result<(u32, u32)> {
    let mut vlqs = vlq::decode_vlqs(item);
    let line = vlqs.next_required()?.unsigned();
    let column = vlqs.next_required()?.unsigned();

    Ok(state.position.advance(line, column))
}

/// Decodes one entry in a sourcemap's `scopes` array, containing
/// scope information corresponding to one particular source file.
///
/// The textual format is a comma-separated list of records. Each record
/// must start with `B`, in which case it starts a scope, or `C`, in which
/// case it ends a previously started scope, or `D`, in which case it
/// contains variable information for the current scope. Variables aren't
/// currently handled.
///
/// Scope trees are encoded by simply nesting `B` and `C` records appropriately.
///
/// The returned struct contains the parsed scopes in post-order in the `all_scopes`
/// field. Each scope's `children` list contains the indices of its children
/// (which precede it) in `all_scopes`. Furthermore, the `roots` field contains
/// the indices of the elements of `all_scopes` which are root scopes.
pub fn decode_scopes(items: &str) -> Result<OriginalScopes> {
    if items.is_empty() {
        return Ok(Default::default());
    }

    // A stack of nested in-progress scopes.
    let mut in_progress = vec![];
    let mut state = DecodeScopeState::default();
    let mut out = OriginalScopes::default();

    for item in items.split(',') {
        match item.split_at_checked(1) {
            Some(("B", start)) => {
                in_progress.push(decode_scope_start(start, &mut state)?);
            }
            Some(("C", end)) => {
                // If there's no scope in progress, we have unbalanced start and end records.
                let mut current = match in_progress.pop() {
                    Some(current) => current,
                    None => return Err(Error::InvalidScopes),
                };

                let (end_line, end_col) = decode_scope_end(end, &mut state)?;
                current.end_line = end_line;
                current.end_col = end_col;

                // The index the current scope will have in the output list.
                let current_idx = out.all_scopes.len();

                // If there's a scope in progress before the current one,
                // we add the current one to its children.
                if let Some(parent) = in_progress.last_mut() {
                    parent.children.push(current_idx);
                }

                // We can now go back over the current scope's children (previously
                // written) and set the parent.
                for &child_idx in &current.children {
                    out.all_scopes[child_idx].parent = Some(current_idx);
                }

                out.all_scopes.push(current);

                // If there's nothing left on the stack, then this was a
                // root scope.
                if in_progress.is_empty() {
                    out.roots.push(current_idx);
                }
            }
            Some(("D", _vars)) => {
                // variables would be handled here
            }
            _ => return Err(Error::InvalidScopes),
        }
    }

    // If the `in_progress` stack isn't empty at the point
    // where we run out of records, we have unbalanced
    // start and end records.
    if !in_progress.is_empty() {
        return Err(Error::InvalidScopes);
    }

    Ok(out)
}
