use lazy_static::lazy_static;
use regex::Regex;
use unicode_id::UnicodeID;

lazy_static! {
    static ref ANCHORED_IDENT_RE: Regex = Regex::new(
        r#"(?x)
            ^
            \s*
            ([\d\p{Lu}\p{Ll}\p{Lt}\p{Lm}\p{Lo}\p{Nl}$_]
            [\d\p{Lu}\p{Ll}\p{Lt}\p{Lm}\p{Lo}\p{Nl}\p{Mn}\p{Mc}\p{Nd}\p{Pc}$_]*)
        "#
    )
    .unwrap();
}

fn is_valid_javascript_identifier_original(s: &str) -> bool {
    // check explicitly we do not have a dot in this identifier so that
    // we do not match on foo.bar
    s.trim() == s && !s.contains('.') && ANCHORED_IDENT_RE.is_match(s)
}

fn get_javascript_token_original(source_line: &str) -> Option<&str> {
    if let Some(m) = ANCHORED_IDENT_RE.captures(source_line) {
        let rng = m.get(1).unwrap();
        Some(&source_line[rng.start()..rng.end()])
    } else {
        None
    }
}

/// Returns true if `c` is a valid character for an identifier start.
fn is_valid_start(c: char) -> bool {
    c == '$' || c == '_' || c.is_ascii_alphabetic() || {
        if c.is_ascii() {
            false
        } else {
            UnicodeID::is_id_start(c)
        }
    }
}

/// Returns true if `c` is a valid character for an identifier part after
/// start.
fn is_valid_continue(c: char) -> bool {
    c == '$' || c == '_' || c == '\u{200c}' || c == '\u{200d}' || c.is_ascii_alphanumeric() || {
        if c.is_ascii() {
            false
        } else {
            UnicodeID::is_id_continue(c)
        }
    }
}

fn strip_to_js_token(s: &str) -> Option<&str> {
    let mut iter = s.char_indices();
    // Is the first character a valid starting character
    match iter.next() {
        Some((_, c)) => {
            if !is_valid_start(c) {
                return None;
            }
        }
        None => {
            return None;
        }
    };
    // Slice up to the last valid continuation character
    let mut end_idx = 0;
    for (i, c) in iter {
        if is_valid_continue(c) {
            end_idx = i;
        } else {
            break;
        }
    }
    return Some(&s[..=end_idx]);
}

pub fn is_valid_javascript_identifier(s: &str) -> bool {
    // check explicitly we do not have a dot in this identifier so that
    // we do not match on foo.bar
    !s.contains('.') && strip_to_js_token(s).map_or(0, |t| t.len()) == s.len()
}

pub fn get_javascript_token(source_line: &str) -> Option<&str> {
    match source_line.split_whitespace().next() {
        Some(s) => strip_to_js_token(s),
        None => None,
    }
}

#[test]
fn test_is_valid_javascript_identifier() {
    assert_eq!(is_valid_javascript_identifier_original("foo 123"), true);
    // assert_eq!(is_valid_javascript_identifier("foo 123"), true);
    assert_eq!(is_valid_javascript_identifier_original("foo_$123"), true);
    assert_eq!(is_valid_javascript_identifier("foo_$123"), true);
    assert_eq!(is_valid_javascript_identifier_original(" foo"), false);
    assert_eq!(is_valid_javascript_identifier(" foo"), false);
    assert_eq!(is_valid_javascript_identifier_original("foo "), false);
    assert_eq!(is_valid_javascript_identifier("foo "), false);
    assert_eq!(is_valid_javascript_identifier_original("[123]"), false);
    assert_eq!(is_valid_javascript_identifier("[123]"), false);
    assert_eq!(is_valid_javascript_identifier_original("foo.bar"), false);
    assert_eq!(is_valid_javascript_identifier("foo.bar"), false);
    // Should these pass?
    assert_eq!(is_valid_javascript_identifier_original("foo [bar]"), true);
    // assert_eq!(is_valid_javascript_identifier("foo [bar]"), true);
    assert_eq!(is_valid_javascript_identifier_original("foo[bar]"), true);
    // assert_eq!(is_valid_javascript_identifier("foo[bar]"), true);

    assert_eq!(get_javascript_token_original("foo "), Some("foo"));
    assert_eq!(get_javascript_token("foo "), Some("foo"));
    assert_eq!(get_javascript_token_original("f _hi"), Some("f"));
    assert_eq!(get_javascript_token("f _hi"), Some("f"));
    assert_eq!(get_javascript_token_original("foo.bar"), Some("foo"));
    assert_eq!(get_javascript_token("foo.bar"), Some("foo"));
    assert_eq!(get_javascript_token_original("[foo,bar]"), None);
    assert_eq!(get_javascript_token("[foo,bar]"), None);
}
