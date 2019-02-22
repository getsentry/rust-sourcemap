use rustc_version::{version, Version};

pub fn main() {
    if version().unwrap() >= Version::parse("1.20.0").unwrap() {
        println!("cargo:rustc-cfg=with_safe_slices");
    }
}
