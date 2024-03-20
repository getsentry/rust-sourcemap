use sourcemap::SourceMapBuilder;

#[test]
fn test_builder_into_sourcemap() {
    let mut builder = SourceMapBuilder::new(None);
    builder.set_source_root(Some("/foo/bar"));
    builder.add_source("baz.js");
    builder.add_name("x");

    let sm = builder.into_sourcemap();
    assert_eq!(sm.get_source_root(), Some("/foo/bar"));
    assert_eq!(sm.get_source(0), Some("/foo/bar/baz.js"));
    assert_eq!(sm.get_name(0), Some("x"));

    let expected = br#"{"version":3,"sources":["baz.js"],"sourceRoot":"/foo/bar","names":["x"],"rangeMappings":"","mappings":""}"#;
    let mut output: Vec<u8> = vec![];
    sm.to_writer(&mut output).unwrap();
    assert_eq!(output, expected);
}
