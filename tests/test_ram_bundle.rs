use sourcemap::{split_ram_bundle, RamBundle, RamBundleModule, SourceMap, SourceMapIndex};
use std::fs;
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
    assert_eq!(ram_bundle.module_count(), 5);
    assert_eq!(ram_bundle.startup_code_size(), 0x7192);
    assert_eq!(ram_bundle.startup_code_offset(), 0x34);

    // Check first modules
    let mut module_iter = ram_bundle.iter_modules();

    let module_0 = module_iter.next().unwrap().unwrap();
    let module_0_data = module_0.data();
    assert_eq!(module_0.id(), 0);
    assert_eq!(module_0_data.len(), 0xa8 - 1);
    assert_eq!(
        module_0_data.get(0..60).unwrap(),
        "__d(function(g,r,i,a,m,e,d){\"use strict\";const o=r(d[0]),s=r".as_bytes()
    );

    let module_3 = module_iter.next().unwrap().unwrap();
    let module_3_data = module_3.data();
    assert_eq!(module_3.id(), 3);
    assert_eq!(module_3_data.len(), 0x6b - 1);
    assert_eq!(
        module_3_data.get(0..60).unwrap(),
        "__d(function(g,r,i,a,m,e,d){\"use strict\";console.log('inside".as_bytes()
    );

    let module_1 = ram_bundle.get_module(1).unwrap();
    assert!(module_1.is_none());

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

    let x_facebook_offsets = ism.x_facebook_offsets().unwrap();
    let x_metro_module_paths = ism.x_metro_module_paths().unwrap();

    assert_eq!(x_facebook_offsets.len(), 367);

    let out_dir = Path::new("out");

    for module in ram_bundle.iter_modules() {
        let rbm = module.unwrap();
        let module_id = rbm.id();
        println!("Checking module with id {}", module_id);
        let out_file = out_dir.join(format!("{}.js", module_id));
        let mut out = File::create(out_file)?;
        let module_data = rbm.data();
        out.write(module_data)?;
    }

    println!("Flattening indexed source map...");
    let sm = ism.flatten().unwrap();
    let out_file = out_dir.join("out.js.map");
    let out = File::create(out_file)?;
    sm.to_writer(out).unwrap();

    let token = sm.lookup_token(367, 1010).unwrap();
    println!("token: {}", token);

    // OUT
    let out_combined = Path::new("out/combined");
    let ram_bundle_iter = split_ram_bundle(&ram_bundle, &ism).unwrap();
    for result in ram_bundle_iter {
        let (name, sv, sm) = result.unwrap();
        println!("name: {}", name);
        let out_sm = File::create(out_combined.join(format!("{}.map", name)))?;
        sm.to_writer(out_sm);

        fs::write(out_combined.join(name.clone()), sv.source())?;
    }

    // TEST
    let sm_data = File::open("out/combined/28.js.map")?;
    let sm = SourceMap::from_reader(sm_data).unwrap();
    let token = sm.lookup_token(0, 2565).unwrap(); // line-number and column
    println!("token: {}", token);
    Ok(())
}
