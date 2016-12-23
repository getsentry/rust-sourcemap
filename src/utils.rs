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


pub fn find_common_prefix<'a, I: Iterator<Item=&'a str>>(iter: I) -> Option<String> {
    let mut items : Vec<Vec<&str>> = iter.map(|x| split_path(x)).collect();
    if items.is_empty() {
        return None;
    }
    items.sort_by_key(|x| x.len());
    let shortest = &items[0];

    let mut max_idx = None;
    for seq in items.iter() {
        let mut seq_max_idx = 0;
        for (idx, &comp) in shortest.iter().enumerate() {
            if seq.get(idx) != Some(&comp) {
                break;
            }
            seq_max_idx = idx;
        }
        if max_idx.is_none() || Some(seq_max_idx) < max_idx {
            max_idx = Some(seq_max_idx);
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
fn test_split_path() {
    assert_eq!(split_path("/foo/bar/baz"), &["", "/foo", "/bar", "/baz"]);
}

#[test]
fn test_find_common_prefix() {
    let rv = find_common_prefix(vec!["/foo/bar/baz", "/foo/bar/baz/blah"].into_iter());
    assert_eq!(rv, Some("/foo/bar/baz".into()));
}
