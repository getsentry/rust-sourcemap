use sourcemap::{is_sourcemap_slice, locate_sourcemap_reference, SourceMapRef};

#[test]
fn test_basic_locate() {
    let input: &[_] = b"foo();\nbar();\n//# sourceMappingURL=foo.js";
    assert_eq!(
        locate_sourcemap_reference(input).unwrap(),
        SourceMapRef::Ref("foo.js".into())
    );
    assert_eq!(
        locate_sourcemap_reference(input).unwrap().get_url(),
        Some("foo.js")
    );
}

#[test]
fn test_legacy_locate() {
    let input: &[_] = b"foo();\nbar();\n//@ sourceMappingURL=foo.js";
    assert_eq!(
        locate_sourcemap_reference(input).unwrap(),
        SourceMapRef::LegacyRef("foo.js".into())
    );
    assert_eq!(
        locate_sourcemap_reference(input).unwrap().get_url(),
        Some("foo.js")
    );
}

#[test]
fn test_no_ref() {
    let input: &[_] = b"foo();\nbar();\n// whatever";
    assert_eq!(
        locate_sourcemap_reference(input).unwrap(),
        SourceMapRef::Missing
    );
}

#[test]
fn test_detect_basic_sourcemap() {
    let input: &[_] = b"{
        \"version\":3,
        \"sources\":[\"coolstuff.js\"],
        \"names\":[\"x\",\"alert\"],
        \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
    }";
    assert!(is_sourcemap_slice(input));
}

#[test]
fn test_detect_bad_sourcemap() {
    let input: &[_] = b"{
        \"sources\":[\"coolstuff.js\"],
        \"names\":[\"x\",\"alert\"]
    }";
    assert!(!is_sourcemap_slice(input));
}

#[test]
fn test_detect_basic_sourcemap_with_junk_header() {
    let input: &[_] = b")]}garbage\n
    {
        \"version\":3,
        \"sources\":[\"coolstuff.js\"],
        \"names\":[\"x\",\"alert\"],
        \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
    }";
    assert!(is_sourcemap_slice(input));
}
