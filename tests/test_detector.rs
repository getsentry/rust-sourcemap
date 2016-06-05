extern crate sourcemap;

use sourcemap::{SourceMapRef, locate_sourcemap_reference};

#[test]
fn test_basic_locate() {
    let input : &[_] = b"foo();\nbar();\n//# sourceMappingURL=foo.js";
    assert_eq!(locate_sourcemap_reference(input).unwrap(),
               SourceMapRef::Ref("foo.js".into()));
    assert_eq!(locate_sourcemap_reference(input).unwrap().get_url(), Some("foo.js"));
}

#[test]
fn test_legacy_locate() {
    let input : &[_] = b"foo();\nbar();\n//@ sourceMappingURL=foo.js";
    assert_eq!(locate_sourcemap_reference(input).unwrap(),
               SourceMapRef::LegacyRef("foo.js".into()));
    assert_eq!(locate_sourcemap_reference(input).unwrap().get_url(), Some("foo.js"));
}

#[test]
fn test_no_ref() {
    let input : &[_] = b"foo();\nbar();\n// whatever";
    assert_eq!(locate_sourcemap_reference(input).unwrap(),
               SourceMapRef::Missing);
}
