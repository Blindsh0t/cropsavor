use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let common_dir = manifest_dir.join("common");
    let library = common_dir.join("libcodec.a");

    if !library.exists() {
        let status = Command::new("make")
            .arg("-C")
            .arg(&common_dir)
            .status()
            .expect("failed to run make for common/libcodec.a");

        assert!(status.success(), "failed to build common/libcodec.a");
    }

    println!("cargo:rustc-link-search=native={}", common_dir.display());
    println!("cargo:rustc-link-lib=static=codec");

    println!("cargo:rerun-if-changed=common/codec.c");
    println!("cargo:rerun-if-changed=common/codec.h");
}
