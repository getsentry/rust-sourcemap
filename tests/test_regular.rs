use sourcemap::SourceMap;

#[test]
fn test_basic_sourcemap() {
    let input: &[_] = br#"{
        "version": 3,
        "sources": ["coolstuff.js"],
        "names": ["x","alert"],
        "mappings": "AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM"
    }"#;
    let sm = SourceMap::from_reader(input).unwrap();

    assert_eq!(
        sm.lookup_token(0, 0).unwrap().to_tuple(),
        ("coolstuff.js", 0, 0, None)
    );
    assert_eq!(
        sm.lookup_token(0, 3).unwrap().to_tuple(),
        ("coolstuff.js", 0, 4, Some("x"))
    );
    assert_eq!(
        sm.lookup_token(0, 24).unwrap().to_tuple(),
        ("coolstuff.js", 2, 8, None)
    );

    // Lines continue out to infinity
    assert_eq!(
        sm.lookup_token(0, 1000).unwrap().to_tuple(),
        ("coolstuff.js", 2, 8, None)
    );

    // Token can return prior lines.
    assert_eq!(
        sm.lookup_token(1000, 0).unwrap().to_tuple(),
        ("coolstuff.js", 2, 8, None)
    );
}
