#[test]
fn test_vlq_decode() {
    use sourcemap::internals::parse_vlq_segment;
    let rv = parse_vlq_segment("AAAA").unwrap();
    assert_eq!(rv, vec![0, 0, 0, 0]);
    let rv = parse_vlq_segment("GAAIA").unwrap();
    assert_eq!(rv, vec![3, 0, 0, 4, 0]);
}

#[test]
fn test_vlq_encode() {
    use sourcemap::internals::generate_vlq_segment;
    let rv = generate_vlq_segment(&[0, 0, 0, 0]).unwrap();
    assert_eq!(rv.as_str(), "AAAA");
    let rv = generate_vlq_segment(&[3, 0, 0, 4, 0]).unwrap();
    assert_eq!(rv.as_str(), "GAAIA");
}

#[test]
fn test_overflow() {
    use sourcemap::internals::parse_vlq_segment;
    use sourcemap::Error;
    match parse_vlq_segment("00000000000000") {
        Err(Error::VlqOverflow) => {}
        e => {
            panic!("Unexpeted result: {:?}", e);
        }
    }
}
