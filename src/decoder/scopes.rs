use crate::types::RawOriginalScope;
use crate::Error::BadSegmentSize;
use crate::{vlq, Error, Result};

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
    /// Buffer for decoding VLQs.
    vlq_buffer: Vec<i64>,
}

impl DecodeScopeState {
    fn decode_vlq(&mut self, segment: &str) -> Result<&[i64]> {
        self.vlq_buffer.clear();
        vlq::parse_vlq_segment_into(segment, &mut self.vlq_buffer)?;
        Ok(&self.vlq_buffer[..])
    }
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

    let nums = state.decode_vlq(item)?;
    if nums.len() < 3 {
        return Err(BadSegmentSize(nums.len() as u32));
    }

    let flags = unsigned(nums[0] as i32);
    let line = unsigned(nums[1] as i32);
    let column = unsigned(nums[2] as i32);

    let has_name = (flags & NAME_FLAG) > 0;
    let has_kind = (flags & KIND_FLAG) > 0;
    let is_stack_frame = (flags & STACKFRAME_FLAG) > 0;

    let (name_id, kind_id) = match (has_name, has_kind) {
        (false, false) => (None, None),
        (true, false) => {
            if nums.len() < 4 {
                return Err(BadSegmentSize(nums.len() as u32));
            }

            (Some(nums[3] as i32), None)
        }
        (false, true) => {
            if nums.len() < 4 {
                return Err(BadSegmentSize(nums.len() as u32));
            }

            (None, Some(nums[3] as i32))
        }
        (true, true) => {
            if nums.len() < 5 {
                return Err(BadSegmentSize(nums.len() as u32));
            }

            (Some(nums[3] as i32), Some(nums[4] as i32))
        }
    };

    let (start_line, start_col) = state.position.advance(line, column);
    let name_id = name_id.map_or(u32::MAX, |name_incr| state.name_idx.add(name_incr));
    let kind_id = kind_id.map_or(u32::MAX, |kind_incr| state.kind_idx.add(kind_incr));

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
    let nums = state.decode_vlq(item)?;
    if nums.len() < 2 {
        return Err(BadSegmentSize(nums.len() as u32));
    }

    let line = unsigned(nums[0] as i32);
    let column = unsigned(nums[1] as i32);

    Ok(state.position.advance(line, column))
}

/// Decodes one entry in a sourcemap's `scopes` array, containing
/// scope information corresponding to one particular source file.
///
/// The textual format is a comma-separated list of entries. Each entry
/// must start with `B`, in which case it starts a scope, or `C`, in which
/// case it ends a previously started scope, or `D`, in which case it
/// contains variable information for the current scope. Variables aren't
/// currently handled.
///
/// Scope trees are encoded by simply nesting `B` and `C` entries appropriately.
///
/// The returned list contains the parsed scopes in post-order. Each scope's
/// `children` list contains the indices of its children (which precede it
/// in the list).
pub fn decode_scopes(items: &str) -> Result<Vec<RawOriginalScope>> {
    // List of in-progress scopes, organized by nesting level.
    //
    // All scopes in `in_progress[j+1]` are children of the last scope in
    // `in_progress[j]`. Scopes in `in_progress[0]` are top-level scopes
    // which will be returned.
    let mut in_progress = vec![vec![]];
    let mut state = DecodeScopeState::default();
    let mut out: Vec<RawOriginalScope> = Vec::new();

    for item in items.split(',') {
        match item.split_at_checked(1) {
            Some(("B", start)) => {
                in_progress
                    .last_mut()
                    .expect("in_progress is never empty")
                    .push(decode_scope_start(start, &mut state)?);

                // Push a new layer for this scope's children.
                in_progress.push(Vec::new());
            }
            Some(("C", end)) => {
                // Pop the children of the scope that's being ended.
                // The layer for the children was added when the scope was started,
                // so if this fails, the scope data is malformed.
                let children = match in_progress.pop() {
                    Some(children) => children,
                    None => return Err(Error::InvalidScopes),
                };

                // The last scope in the last layer (after popping the children) is the scope which is being closed by this end record.
                // If it doesn't exist, the end record has no corresponding start record, so the scope data is malformed.
                let current = match in_progress.last_mut().and_then(|level| level.last_mut()) {
                    Some(current) => current,
                    None => return Err(Error::InvalidScopes),
                };

                let (end_line, end_col) = decode_scope_end(end, &mut state)?;
                current.end_line = end_line;
                current.end_col = end_col;

                // Append the current scope's children to the output list and
                // record their indices in the current scope.
                for child in children {
                    for &grandchild_idx in &child.children {
                        out[grandchild_idx].parent = Some(out.len());
                    }
                    current.children.push(out.len());
                    out.push(child);
                }
            }
            Some(("D", _vars)) => {
                // variables would be handled here
            }
            _ => return Err(Error::InvalidScopes),
        }
    }

    // When we run out of items, exactly one layer of scopes (the top-level ones) should remain.
    // Anything else indicates malformed data.
    if in_progress.len() != 1 {
        return Err(Error::InvalidScopes);
    }

    for scope in in_progress.remove(0) {
        for &child_idx in &scope.children {
            out[child_idx].parent = Some(out.len());
        }
        out.push(scope);
    }

    Ok(out)
}

/// Reinterprets a signed VLQ as an unsigned number.
///
/// This reverts the operation described in
/// <https://tc39.es/ecma426/branch/proposal-scopes/#sec-VLQSignedValue>.
fn unsigned(mut vlq: i32) -> u32 {
    if vlq == i32::MIN {
        return 1;
    }

    let neg = vlq < 0;

    if neg {
        vlq = -vlq;
    }

    vlq <<= 1;

    if neg {
        vlq |= 1;
    }

    vlq as u32
}

#[test]
fn test_decode_scopes() {
    let input = "BCCAQ,BHAAAC,BCCaC,CDC,CCA,CCY";

    let scopes = decode_scopes(input).unwrap();
    dbg!(scopes);
    panic!();
}
