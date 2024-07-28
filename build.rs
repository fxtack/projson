use std::process::Command;

fn main () {
    if cfg!(target_os = "windows") {
        embed_resource::compile("./projson.rc");
    } else {
        panic!("this crate can only be compiled on Windows");
    }

    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let mut build_hash = "0".to_string();
    if output.status.success() {
        build_hash = String::from_utf8(output.stdout).unwrap()[0..8].trim_end().to_string();
    }
    println!("cargo:rustc-env=BUILD_HASH={}", build_hash)
}