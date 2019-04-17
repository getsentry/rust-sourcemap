use std::io;
use std::io::BufRead;

use sourcemap::internals::StripHeaderReader;
use sourcemap::{decode_data_url, DecodedMap, SourceMap, Token};

#[test]
fn test_strip_header() {
    let input: &[_] = b")]}garbage\r\n[1, 2, 3]";
    let mut reader = io::BufReader::new(StripHeaderReader::new(input));
    let mut text = String::new();
    reader.read_line(&mut text).ok();
    assert_eq!(text, "[1, 2, 3]");
}

#[test]
fn test_no_header() {
    let input: &[_] = b"[1, 2, 3]";
    let mut reader = io::BufReader::new(StripHeaderReader::new(input));
    let mut text = String::new();
    reader.read_line(&mut text).ok();
    assert_eq!(text, "[1, 2, 3]");
}

#[test]
fn test_no_header_object() {
    let input: &[_] = b"{\"x\":[1, 2, 3]}";
    let mut reader = io::BufReader::new(StripHeaderReader::new(input));
    let mut text = String::new();
    reader.read_line(&mut text).ok();
    assert_eq!(text, "{\"x\":[1, 2, 3]}");
}

#[test]
fn test_bad_newline() {
    let input: &[_] = b")]}'\r[1, 2, 3]";
    let mut reader = io::BufReader::new(StripHeaderReader::new(input));
    let mut text = String::new();
    match reader.read_line(&mut text) {
        Err(err) => {
            assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        }
        Ok(_) => {
            panic!("Expected failure");
        }
    }
}

#[test]
fn test_basic_sourcemap() {
    let input: &[_] = b"{
        \"version\":3,
        \"sources\":[\"coolstuff.js\"],
        \"names\":[\"x\",\"alert\"],
        \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
    }";
    let sm = SourceMap::from_reader(input).unwrap();
    let mut iter = sm.tokens().filter(Token::has_name);
    assert_eq!(
        iter.next().unwrap().to_tuple(),
        ("coolstuff.js", 0, 4, Some("x"))
    );
    assert_eq!(
        iter.next().unwrap().to_tuple(),
        ("coolstuff.js", 1, 4, Some("x"))
    );
    assert_eq!(
        iter.next().unwrap().to_tuple(),
        ("coolstuff.js", 2, 2, Some("alert"))
    );
    assert!(iter.next().is_none());
}

#[test]
fn test_basic_sourcemap_with_root() {
    let input: &[_] = b"{
        \"version\":3,
        \"sources\":[\"coolstuff.js\"],
        \"sourceRoot\":\"x\",
        \"names\":[\"x\",\"alert\"],
        \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
    }";
    let sm = SourceMap::from_reader(input).unwrap();
    let mut iter = sm.tokens().filter(Token::has_name);
    assert_eq!(
        iter.next().unwrap().to_tuple(),
        ("x/coolstuff.js", 0, 4, Some("x"))
    );
    assert_eq!(
        iter.next().unwrap().to_tuple(),
        ("x/coolstuff.js", 1, 4, Some("x"))
    );
    assert_eq!(
        iter.next().unwrap().to_tuple(),
        ("x/coolstuff.js", 2, 2, Some("alert"))
    );
    assert!(iter.next().is_none());
}

#[test]
fn test_sourcemap_data_url() {
    let url = "data:application/json;base64,\
               eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbImNvb2xzdHVmZi5qcyJdLCJzb3VyY2VSb290I\
               joieCIsIm5hbWVzIjpbIngiLCJhbGVydCJdLCJtYXBwaW5ncyI6IkFBQUEsR0FBSUEsR0\
               FBSSxFQUNSLElBQUlBLEdBQUssRUFBRyxDQUNWQyxNQUFNIn0=";
    match decode_data_url(url).unwrap() {
        DecodedMap::Regular(sm) => {
            let mut iter = sm.tokens().filter(Token::has_name);
            assert_eq!(
                iter.next().unwrap().to_tuple(),
                ("x/coolstuff.js", 0, 4, Some("x"))
            );
            assert_eq!(
                iter.next().unwrap().to_tuple(),
                ("x/coolstuff.js", 1, 4, Some("x"))
            );
            assert_eq!(
                iter.next().unwrap().to_tuple(),
                ("x/coolstuff.js", 2, 2, Some("alert"))
            );
            assert!(iter.next().is_none());
        }
        _ => {
            panic!("did not get sourcemap");
        }
    }
}

#[test]
fn test_sourcemap_nofiles() {
    let input: &[_] = b"{
        \"version\":3,
        \"sources\":[null],
        \"names\":[\"x\",\"alert\"],
        \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
    }";
    let sm = SourceMap::from_reader(input).unwrap();
    let mut iter = sm.tokens().filter(Token::has_name);
    assert_eq!(iter.next().unwrap().to_tuple(), ("", 0, 4, Some("x")));
    assert_eq!(iter.next().unwrap().to_tuple(), ("", 1, 4, Some("x")));
    assert_eq!(iter.next().unwrap().to_tuple(), ("", 2, 2, Some("alert")));
    assert!(iter.next().is_none());
}
