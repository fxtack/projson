use std::process::Command;

fn main () {
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let mut build_hash = "0".to_string();
    if output.status.success() {
        build_hash = String::from_utf8(output.stdout).unwrap()[0..8].trim_end().to_string();
    }
    println!("cargo:rustc-env=BUILD_HASH={}", build_hash)
}