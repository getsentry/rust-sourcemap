use sourcemap::{split_ram_bundle, RamBundle, SourceMapIndex};
use std::fs::File;
use std::io::Read;

#[test]
fn test_basic_ram_bundle_parse() -> Result<(), Box<std::error::Error>> {
    let mut bundle_file = File::open("./tests/fixtures/ram_bundle/basic.jsbundle")?;
    let mut bundle_data = Vec::new();
    bundle_file.read_to_end(&mut bundle_data)?;
    let ram_bundle = RamBundle::parse(&bundle_data)?;

    // Header checks
    assert_eq!(ram_bundle.module_count(), 5);
    assert_eq!(ram_bundle.startup_code_size(), 0x7192);
    assert_eq!(ram_bundle.startup_code_offset(), 0x34);

    // Check first modules
    let mut module_iter = ram_bundle.iter_modules();

    let module_0 = module_iter.next().unwrap()?;
    let module_0_data = module_0.data();
    assert_eq!(module_0.id(), 0);
    assert_eq!(module_0_data.len(), 0xa8 - 1);
    assert_eq!(
        module_0_data.get(0..60).unwrap(),
        "__d(function(g,r,i,a,m,e,d){\"use strict\";const o=r(d[0]),s=r".as_bytes()
    );

    let module_3 = module_iter.next().unwrap()?;
    let module_3_data = module_3.data();
    assert_eq!(module_3.id(), 3);
    assert_eq!(module_3_data.len(), 0x6b - 1);
    assert_eq!(
        module_3_data.get(0..60).unwrap(),
        "__d(function(g,r,i,a,m,e,d){\"use strict\";console.log('inside".as_bytes()
    );

    let module_1 = ram_bundle.get_module(1)?;
    assert!(module_1.is_none());

    Ok(())
}

#[test]
fn test_basic_ram_bundle_split() -> Result<(), Box<std::error::Error>> {
    let mut bundle_file = File::open("./tests/fixtures/ram_bundle/basic.jsbundle")?;
    let mut bundle_data = Vec::new();
    bundle_file.read_to_end(&mut bundle_data)?;
    let ram_bundle = RamBundle::parse(&bundle_data)?;

    let sourcemap_file = File::open("./tests/fixtures/ram_bundle/basic.jsbundle.map")?;
    let ism = SourceMapIndex::from_reader(sourcemap_file)?;

    assert!(ism.is_for_react_native());

    let x_facebook_offsets = ism.x_facebook_offsets().unwrap();
    assert_eq!(x_facebook_offsets.len(), 5);

    let x_metro_module_paths = ism.x_metro_module_paths().unwrap();
    assert_eq!(x_metro_module_paths.len(), 7);

    // Modules 0, 3, 4
    assert_eq!(split_ram_bundle(&ram_bundle, &ism)?.count(), 3);

    let mut ram_bundle_iter = split_ram_bundle(&ram_bundle, &ism)?;

    let (name, sourceview, sourcemap) = ram_bundle_iter.next().unwrap()?;
    assert_eq!(name, "0.js");
    assert_eq!(
        &sourceview.source()[0..60],
        "__d(function(g,r,i,a,m,e,d){\"use strict\";const o=r(d[0]),s=r"
    );
    assert_eq!(
        &sourcemap.get_source_contents(0).unwrap()[0..60],
        "const f = require(\"./other\");\nconst isWindows = require(\"is-"
    );

    Ok(())
}
