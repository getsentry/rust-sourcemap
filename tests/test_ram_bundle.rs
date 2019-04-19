use sourcemap::RamBundle;
use std::fs::File;
use std::io::Read;

#[test]
fn test_basic_ram_bundle() {
    let mut bundle_file = File::open("./tests/fixtures/basic.jsbundle").unwrap();
    let mut bundle_data = Vec::new();
    bundle_file.read_to_end(&mut bundle_data);
    let ram_bundle = RamBundle::parse(&bundle_data).unwrap();

    assert_eq!(ram_bundle.module_count(), 367);
    assert_eq!(ram_bundle.startup_code_size(), 0x32d2);
    assert_eq!(ram_bundle.startup_code_offset(), 0xb84);

    for module_entry in ram_bundle.iter_modules().take(3) {
        let module = module_entry.unwrap();
        println!("{}", module.unwrap().id());
    }
}
