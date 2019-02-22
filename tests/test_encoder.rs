use sourcemap::SourceMap;

#[test]
fn test_basic_sourcemap() {
    let input: &[_] = b"{
        \"version\":3,
        \"sources\":[\"coolstuff.js\"],
        \"names\":[\"x\",\"alert\"],
        \"mappings\":\"AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM\"
    }";
    let sm = SourceMap::from_reader(input).unwrap();
    let mut out: Vec<u8> = vec![];
    sm.to_writer(&mut out).unwrap();

    let sm2 = SourceMap::from_reader(&out[..]).unwrap();

    for (tok1, tok2) in sm.tokens().zip(sm2.tokens()) {
        assert_eq!(tok1, tok2);
    }
}
