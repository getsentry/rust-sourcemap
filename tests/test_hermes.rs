use sourcemap::SourceMapHermes;

#[test]
fn test_react_native_hermes() {
    let input = include_bytes!("./fixtures/react-native-hermes/output.map");
    let sm = SourceMapHermes::from_reader(&input[..]).unwrap();

    // and these point to the expression I guess, but in the case
    // of throw, its a bit vague.

    //    at foo (address at unknown:1:11939)
    assert_eq!(
        sm.lookup_token(0, 11939).unwrap().to_tuple(),
        ("module.js", 1, 10, None)
    );
    //     throw new Error("lets throw!");
    //           ^

    // at anonymous (address at unknown:1:11857)
    assert_eq!(
        sm.lookup_token(0, 11857).unwrap().to_tuple(),
        ("input.js", 2, 0, None)
    );
    // foo();
    // ^
}
