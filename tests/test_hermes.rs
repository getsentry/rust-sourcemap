use sourcemap::SourceMapHermes;

#[test]
fn test_react_native_hermes() {
    let input = include_bytes!("./fixtures/react-native-hermes/output.map");
    let sm = SourceMapHermes::from_reader(&input[..]).unwrap();

    //    at foo (address at unknown:1:11939)
    assert_eq!(
        sm.lookup_token(0, 11939).unwrap().to_tuple(),
        ("module.js", 1, 10, None)
    );
    assert_eq!(sm.get_original_function_name(11939), Some("foo"));

    // at anonymous (address at unknown:1:11857)
    assert_eq!(
        sm.lookup_token(0, 11857).unwrap().to_tuple(),
        ("input.js", 2, 0, None)
    );
    assert_eq!(sm.get_original_function_name(11857), Some("<global>"));
}

#[test]
fn test_react_native_metro() {
    let input = include_bytes!("./fixtures/react-native-metro/output.js.map");
    let sm = SourceMapHermes::from_reader(&input[..]).unwrap();

    //    at foo (output.js:1289:11)
    let token = sm.lookup_token(1288, 10).unwrap();
    assert_eq!(token.to_tuple(), ("module.js", 1, 10, None));
    assert_eq!(sm.get_scope_for_token(token), Some("foo"));

    // at output.js:1280:19
    let token = sm.lookup_token(1279, 18).unwrap();
    assert_eq!(token.to_tuple(), ("input.js", 2, 0, None));
    assert_eq!(sm.get_scope_for_token(token), Some("<global>"));
}
