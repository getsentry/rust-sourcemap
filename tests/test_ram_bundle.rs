use sourcemap::{RamBundle, RamBundleModule, SourceMapIndex};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

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
    assert_eq!(module_0_data.len(), 0x15b - 1);
    assert_eq!(
        module_0_data.get(0..40).unwrap(),
        "__d(function(g,r,i,a,m,e,d){'use strict'".as_bytes()
    );

    let module_1 = module_iter.next().unwrap().unwrap();
    let module_1_data = module_1.data();
    assert_eq!(module_1.id(), 1);
    assert_eq!(module_1_data.len(), 0xa5 - 1);
    assert_eq!(
        module_1_data.get(0..40).unwrap(),
        "__d(function(g,r,i,a,m,e,d){var n=r(d[0]".as_bytes()
    );

    let module_2 = module_iter.next().unwrap();
    assert!(module_2.is_none());

    Ok(())
}

#[test]
fn test_basic_ram_bundle_with_sourcemap() -> Result<(), std::io::Error> {
    let mut bundle_file = File::open("./tests/fixtures/ram_bundle/basic.jsbundle").unwrap();
    let mut bundle_data = Vec::new();
    bundle_file.read_to_end(&mut bundle_data)?;
    let ram_bundle = RamBundle::parse(&bundle_data).unwrap();

    let mut sourcemap_file = File::open("./tests/fixtures/ram_bundle/ios.bundle.map").unwrap();
    let ism = SourceMapIndex::from_reader(sourcemap_file).unwrap();

    assert!(ism.is_for_react_native());

    let x_facebook_offsets = ism.x_facebook_offsets().as_ref().unwrap();
    let x_metro_module_paths = ism.x_metro_module_paths().as_ref().unwrap();

    assert_eq!(x_facebook_offsets.len(), 367);

    let out_dir = Path::new("out");

    for module in ram_bundle.iter_modules() {
        match module {
            Some(rbm) => {
                let module_id = rbm.id();
                println!("Checking module with id {}", module_id);
                let out_file = out_dir.join(format!("{}.js", module_id));
                let mut out = File::create(out_file)?;
                let module_data = rbm.data();
                out.write(module_data)?;
            }
            None => println!("skipping module"),
        }
    }

    println!("Flattening indexed source map...");
    let sm = ism.flatten().unwrap();
    let out_file = out_dir.join("out.js.map");
    let out = File::create(out_file)?;
    sm.to_writer(out).unwrap();

    let token = sm.lookup_token(368, 1010).unwrap();
    println!("token: {}", token);

    Ok(())
}
