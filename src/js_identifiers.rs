/// Returns true if `c` is a valid character for an identifier start.
fn is_valid_start(c: char) -> bool {
    c == '$' || c == '_' || c.is_ascii_alphabetic() || {
        if c.is_ascii() {
            false
        } else {
            unicode_id_start::is_id_start_unicode(c)
        }
    }
}

/// Returns true if `c` is a valid character for an identifier part after start.
fn is_valid_continue(c: char) -> bool {
    // As specified by the ECMA-262 spec, U+200C (ZERO WIDTH NON-JOINER) and U+200D
    // (ZERO WIDTH JOINER) are format-control characters that are used to make necessary
    // distinctions when forming words or phrases in certain languages. They are however
    // not considered by UnicodeID to be universally valid identifier characters.
    c == '$' || c == '_' || c == '\u{200c}' || c == '\u{200d}' || c.is_ascii_alphanumeric() || {
        if c.is_ascii() {
            false
        } else {
            unicode_id_start::is_id_continue_unicode(c)
        }
    }
}

fn strip_identifier(s: &str) -> Option<&str> {
    let mut iter = s.char_indices();
    // Is the first character a valid starting character
    let first_char_len = match iter.next() {
        Some((_, c)) => {
            if !is_valid_start(c) {
                return None;
            }
            c.len_utf8()
        }
        None => {
            return None;
        }
    };
    // Slice up to the last valid continuation character
    // Initialize to end of first char to handle single-char and multibyte identifiers correctly
    let mut end_idx = first_char_len;
    for (i, c) in iter {
        if is_valid_continue(c) {
            // Store the end byte index (start + char length) for proper UTF-8 boundary
            end_idx = i + c.len_utf8();
        } else {
            break;
        }
    }
    Some(&s[..end_idx])
}

pub fn is_valid_javascript_identifier(s: &str) -> bool {
    // check stripping does not reduce the length of the token
    strip_identifier(s).map_or(0, |t| t.len()) == s.len()
}

/// Finds the first valid identifier in the JS Source string given, provided
/// the string begins with the identifier or whitespace.
pub fn get_javascript_token(source_line: &str) -> Option<&str> {
    match source_line.split_whitespace().next() {
        Some(s) => strip_identifier(s),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_javascript_identifier() {
        // assert_eq!(is_valid_javascript_identifier("foo 123"));
        assert!(is_valid_javascript_identifier("foo_$123"));
        assert!(!is_valid_javascript_identifier(" foo"));
        assert!(!is_valid_javascript_identifier("foo "));
        assert!(!is_valid_javascript_identifier("[123]"));
        assert!(!is_valid_javascript_identifier("foo.bar"));

        // Non-ASCII identifiers
        assert!(is_valid_javascript_identifier("한글변수"));
        assert!(is_valid_javascript_identifier("变量名"));
        assert!(is_valid_javascript_identifier("ひらがな"));

        // Should these pass?
        // assert!(is_valid_javascript_identifier("foo [bar]"));
        assert_eq!(get_javascript_token("foo "), Some("foo"));
        assert_eq!(get_javascript_token("f _hi"), Some("f"));
        assert_eq!(get_javascript_token("foo.bar"), Some("foo"));
        assert_eq!(get_javascript_token("[foo,bar]"), None);
        assert_eq!(get_javascript_token("결제사_연결():De"), Some("결제사_연결"));
        assert_eq!(get_javascript_token("变量名123"), Some("变量名123"));
        assert_eq!(get_javascript_token("へんすう_test"), Some("へんすう_test"));
    }
}
