use std::str::Lines;
use std::iter::Fuse;
use std::cell::RefCell;

use types::{Token, idx_from_token, sourcemap_from_token};
use utils::{get_javascript_token, is_valid_javascript_identifier};


/// An iterator that iterates over tokens in reverse.
pub struct RevTokenIter<'view, 'map> {
    sv: &'view MinifiedSourceView<'view>,
    token: Option<Token<'map>>,
    source_line: Option<(&'view str, usize, usize, usize)>,
}

impl<'view, 'map> Iterator for RevTokenIter<'view, 'map> {
    type Item = (Token<'map>, Option<&'view str>);

    fn next(&mut self) -> Option<(Token<'map>, Option<&'view str>)> {
        let token = match self.token.take() {
            None => { return None; }
            Some(token) => token
        };

        let idx = idx_from_token(&token);
        if idx > 0 {
            let sm = sourcemap_from_token(&token);
            self.token = sm.get_token(idx - 1);
        }

        // if we are going to the same line as we did last iteration, we don't have to scan
        // up to it again.  For normal sourcemaps this should mean we only ever go to the
        // line once.
        let (source_line, last_char_offset, last_byte_offset) = if_chain! {
            if let Some((source_line, dst_line, last_char_offset,
                         last_byte_offset)) = self.source_line;

            if dst_line == token.get_dst_line() as usize;
            then {
                (source_line, last_char_offset, last_byte_offset)
            } else {
                if let Some(source_line) = self.sv.get_line(token.get_dst_line()) {
                    (source_line, !0, !0)
                } else {
                    // if we can't find the line, return am empty one
                    ("", !0, !0)
                }
            }
        };

        // find the byte offset where our token starts
        let byte_offset = if last_byte_offset == !0 {
            let mut off = 0;
            let mut idx = 0;
            for c in source_line.chars() {
                if idx >= token.get_dst_col() as usize {
                    break;
                }
                off += c.len_utf8();
                idx += c.len_utf16();
            }
            off
        } else {
            let chars_to_move = last_char_offset - token.get_dst_col() as usize;
            let mut new_offset = last_byte_offset;
            let mut idx = 0;
            for c in source_line.get(..last_byte_offset).unwrap_or("").chars().rev() {
                if idx >= chars_to_move {
                    break;
                }
                new_offset -= c.len_utf8();
                idx += c.len_utf16();
            }
            new_offset
        };

        // remember where we were
        self.source_line = Some((
            source_line,
            token.get_dst_line() as usize,
            token.get_dst_col() as usize,
            byte_offset,
        ));

        // in case we run out of bounds here we reset the cache
        if byte_offset >= source_line.len() {
            self.source_line = None;
            Some((token, None))
        } else {
            Some((
                token,
                source_line.get(byte_offset..)
                    .and_then(|s| get_javascript_token(s))
            ))
        }
    }
}

/// Provides efficient access to minified sources.
///
/// This type is used to implement farily efficient source mapping
/// operations.
pub struct MinifiedSourceView<'a> {
    source: &'a str,
    source_iter: RefCell<Fuse<Lines<'a>>>,
    lines: RefCell<Vec<&'a str>>,
}

impl<'a> MinifiedSourceView<'a> {
    /// Creates an optimized view of a given minified source.
    pub fn new(source: &'a str) -> MinifiedSourceView<'a> {
        MinifiedSourceView {
            source: source,
            source_iter: RefCell::new(source.lines().fuse()),
            lines: RefCell::new(vec![]),
        }
    }

    /// Returns a requested minified line.
    pub fn get_line(&self, idx: u32) -> Option<&'a str> {
        let idx = idx as usize;
        {
            let lines = self.lines.borrow();
            if idx < lines.len() {
                return Some(lines[idx]);
            }
        }

        let mut source_iter = self.source_iter.borrow_mut();
        let mut lines = self.lines.borrow_mut();
        while let Some(item) = source_iter.next() {
            lines.push(item);
            if lines.len() - 1 >= idx {
                return Some(lines[lines.len() - 1]);
            }
        }

        None
    }

    /// Returns the source.
    pub fn source(&self) -> &str {
        self.source
    }

    fn rev_token_iter<'map>(&'a self, token: Token<'map>)
        -> RevTokenIter<'a, 'map>
    {
        RevTokenIter {
            sv: self,
            token: Some(token),
            source_line: None,
        }
    }

    /// Given a token and minified function name this attemps to resolve the
    /// name to an original function name.
    ///
    /// This invokes some guesswork and requires access to the original minified
    /// source.  This will not yield proper results for anonymous functions or
    /// functions that do not have clear function names.  (For instance it's
    /// recommended that dotted function names are not passed to this
    /// function).
    pub fn get_original_function_name<'map>(&'a self, token: Token<'map>, minified_name: &str)
        -> Option<&'map str>
    {
        if !is_valid_javascript_identifier(minified_name) {
            return None;
        }

        let mut iter = self.rev_token_iter(token).take(128).peekable();

        while let Some((token, original_identifier)) = iter.next() {
            if_chain! {
                if original_identifier == Some(minified_name);
                if let Some(item) = iter.peek();
                if item.1 == Some("function");
                then {
                    return token.get_name();
                }
            }
        }

        None
    }

    /// Returns the number of lines.
    pub fn line_count(&self) -> usize {
        self.get_line(!0);
        self.lines.borrow().len()
    }
}

#[test]
fn test_minified_source_view() {
    let view = MinifiedSourceView::new("a\nb\nc");
    assert_eq!(view.get_line(0), Some("a"));
    assert_eq!(view.get_line(0), Some("a"));
    assert_eq!(view.get_line(2), Some("c"));
    assert_eq!(view.get_line(1), Some("b"));
    assert_eq!(view.get_line(3), None);

    assert_eq!(view.line_count(), 3);
}
