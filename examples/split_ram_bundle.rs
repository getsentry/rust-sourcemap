use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use sourcemap::{split_ram_bundle, RamBundle, SourceMapIndex};

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 4 {
        panic!("Usage: ./split_ram_bundle RAM_BUNDLE SOURCEMAP OUT_DIRECTORY");
    }

    let mut bundle_file = File::open(&args[1])?;
    let mut bundle_data = Vec::new();
    bundle_file.read_to_end(&mut bundle_data)?;
    let ram_bundle = RamBundle::parse(&bundle_data).unwrap();

    let sourcemap_file = File::open(&args[2])?;
    let ism = SourceMapIndex::from_reader(sourcemap_file).unwrap();

    let output_directory = Path::new(&args[3]);
    if !output_directory.exists() {
        panic!("Directory {} does not exist!", output_directory.display());
    }

    println!(
        "Ouput directory: {}",
        output_directory.canonicalize()?.display()
    );
    let ram_bundle_iter = split_ram_bundle(&ram_bundle, &ism).unwrap();
    for result in ram_bundle_iter {
        let (name, sv, sm) = result.unwrap();
        println!("Writing down source: {}", name);
        fs::write(output_directory.join(name.clone()), sv.source())?;

        let sourcemap_name = format!("{}.map", name);
        println!("Writing down sourcemap: {}", sourcemap_name);
        let out_sm = File::create(output_directory.join(sourcemap_name))?;
        sm.to_writer(out_sm)?;
    }

    Ok(())
}
