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

#[test]
fn test_basic_range() {
    let input: &[_] = br#"{
        "version": 3,
        "sources": ["input.js"],
        "names": ["console","log","ab"],
        "mappings": "AACAA,QAAQC,GAAG,CAAC,OAAM,OAAM,QACxBD,QAAQC,GAAG,CAAC,QAEZD,QAAQC,GAAG,CAJD;IAACC,IAAI;AAAI,IAKnBF,QAAQC,GAAG,CAAC,YACZD,QAAQC,GAAG,CAAC",
        "rangeMappings": "AAB;;g"
    }"#;
    let sm = SourceMap::from_reader(input).unwrap();

    assert_eq!(
        sm.lookup_token(1, 1).unwrap().to_tuple(),
        ("input.js", 2, 2, None)
    );
    assert_eq!(
        sm.lookup_token(1, 8).unwrap().to_tuple(),
        ("input.js", 2, 10, None)
    );
    assert_eq!(
        sm.lookup_token(1, 12).unwrap().to_tuple(),
        ("input.js", 2, 14, None)
    );
}
