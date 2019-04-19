use sourcemap::RamBundle;
use std::fs::File;
use std::io::Read;

#[test]
fn test_basic_ram_bundle() -> Result<(), std::io::Error> {
    let mut bundle_file = File::open("./tests/fixtures/ram_bundle/basic.jsbundle").unwrap();
    let mut bundle_data = Vec::new();
    bundle_file.read_to_end(&mut bundle_data)?;
    let ram_bundle = RamBundle::parse(&bundle_data).unwrap();

    // Header checks
    assert_eq!(ram_bundle.module_count(), 367);
    assert_eq!(ram_bundle.startup_code_size(), 0x32d2);
    assert_eq!(ram_bundle.startup_code_offset(), 0xb84);

    // Check first modules
    let mut module_iter = ram_bundle.iter_modules();

    let module_0 = module_iter.next().unwrap().unwrap();
    let module_0_data = module_0.data();
    assert_eq!(module_0.id(), 0);
    assert_eq!(module_0_data.len(), 0x15b);
    assert_eq!(
        module_0_data.get(0..40).unwrap(),
        "__d(function(g,r,i,a,m,e,d){'use strict'".as_bytes()
    );

    let module_1 = module_iter.next().unwrap().unwrap();
    let module_1_data = module_1.data();
    assert_eq!(module_1.id(), 1);
    assert_eq!(module_1_data.len(), 0xa5);
    assert_eq!(
        module_1_data.get(0..40).unwrap(),
        "__d(function(g,r,i,a,m,e,d){var n=r(d[0]".as_bytes()
    );

    let module_2 = module_iter.next().unwrap();
    assert!(module_2.is_none());

    Ok(())
}
