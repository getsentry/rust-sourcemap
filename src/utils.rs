fn split_path(path: &str) -> Vec<&str> {
    let mut last_idx = 0;
    let mut rv = vec![];
    for (idx, _) in path.match_indices(&['/', '\\'][..]) {
        rv.push(&path[last_idx..idx]);
        last_idx = idx;
    }
    if last_idx < path.len() {
        rv.push(&path[last_idx..]);
    }
    rv
}

fn is_abs_path(s: &str) -> bool {
    if s.starts_with('/') {
        return true;
    } else if s.len() > 3 {
        let b = s.as_bytes();
        if b[1] == b':' && (b[2] == b'/' || b[2] == b'\\') &&
           ((b[0] >= b'a' && b[0] <= b'z') || (b[0] >= b'A' && b[0] <= b'Z')) {
            return true;
        }
    }
    false
}


pub fn find_common_prefix<'a, I: Iterator<Item = &'a str>>(iter: I) -> Option<String> {
    // TODO: this technically does not really do the same thing on windows
    // but honestly, i don't care that much about it right now.
    let mut items: Vec<Vec<&str>> = iter.filter(|x| is_abs_path(x))
        .map(|x| split_path(x))
        .collect();
    if items.is_empty() {
        return None;
    }
    items.sort_by_key(|x| x.len());
    let shortest = &items[0];

    let mut max_idx = None;
    for seq in items.iter() {
        let mut seq_max_idx = None;
        for (idx, &comp) in shortest.iter().enumerate() {
            if seq.get(idx) != Some(&comp) {
                break;
            }
            seq_max_idx = Some(idx);
        }
        if max_idx.is_none() || seq_max_idx < max_idx {
            max_idx = seq_max_idx;
        }
    }

    if let Some(max_idx) = max_idx {
        let rv = shortest[..max_idx + 1].join("");
        if !rv.is_empty() && &rv != "/" {
            return Some(rv);
        }
    }

    None
}


#[test]
fn test_is_abs_path() {
    assert_eq!(is_abs_path("C:\\foo.txt"), true);
    assert_eq!(is_abs_path("d:/foo.txt"), true);
    assert_eq!(is_abs_path("foo.txt"), false);
    assert_eq!(is_abs_path("/foo.txt"), true);
    assert_eq!(is_abs_path("/"), true);
}

#[test]
fn test_split_path() {
    assert_eq!(split_path("/foo/bar/baz"), &["", "/foo", "/bar", "/baz"]);
}

#[test]
fn test_find_common_prefix() {
    let rv = find_common_prefix(vec!["/foo/bar/baz", "/foo/bar/baz/blah"].into_iter());
    assert_eq!(rv, Some("/foo/bar/baz".into()));

    let rv = find_common_prefix(vec!["/foo/bar/baz", "/foo/bar/baz/blah", "/meh"].into_iter());
    assert_eq!(rv, None);

    let rv = find_common_prefix(vec!["/foo/bar/baz", "/foo/bar/baz/blah", "/foo"].into_iter());
    assert_eq!(rv, Some("/foo".into()));

    let rv = find_common_prefix(vec!["/foo/bar/baz", "/foo/bar/baz/blah", "foo"].into_iter());
    assert_eq!(rv, Some("/foo/bar/baz".into()));

    let rv = find_common_prefix(vec!["/foo/bar/baz", "/foo/bar/baz/blah", "/blah", "foo"]
        .into_iter());
    assert_eq!(rv, None);
}
