//! src: https://blog.biofan.org/2019/08/cargo-build-script/
//! 动态生成版本号

use std::env::var;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn get_git_version() -> String {
    let version = var("CARGO_PKG_VERSION").unwrap();

    let child = Command::new("git").args(&["describe", "--always"]).output();
    match child {
        Ok(child) => {
            let buf = String::from_utf8(child.stdout).expect("failed to read stdout");
            format!("v{}\ngit: {}", version, buf)
        }
        Err(err) => {
            eprintln!("`git describe` err: {}", err);
            version.to_string()
        }
    }
}

fn main() {
    let version = get_git_version();
    let p = Path::new(&var("OUT_DIR").unwrap()).join("VERSION");
    let mut f = File::create(p).unwrap();
    f.write_all(version.trim().as_bytes()).unwrap();
}
