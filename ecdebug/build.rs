use std::process::Command;
use std::ffi::OsStr;
use std::path::Path;
use std::env;
fn main() {
    println!("cargo:rerun-if-changed=src/cmd.a51");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out = Path::new(&out_dir);
    Command::new("asem")
            .args(&[OsStr::new("src/cmd.a51"), out.join("cmd.hex").as_os_str(), out.join("cmd.lst").as_os_str()])
            .spawn()
            .expect("Could not run asem, is asem51 installed?");
    Command::new("hexbin")
            .args(&[out.join("cmd.hex")])
            .spawn()
            .expect("Could not run asem, is asem51 installed?");
}