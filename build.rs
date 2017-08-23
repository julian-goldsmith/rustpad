use std::process::Command;
use std::env;
use std::path::Path;
use std::fs;

fn main() {
    let out_dir = env::var("OUT_DIR").ok().expect("can't find out_dir");
	
    Command::new("rc").args(&["/fo", &format!("{}/rustpad.res", out_dir), "src/rustpad.rc"])
                       .status().unwrap();
		
	fs::remove_file(&Path::new(&format!("{}/rustpad_rc.lib", out_dir)));
		
	fs::copy(&Path::new(&format!("{}/rustpad.res", out_dir)), &Path::new(&format!("{}/rustpad_rc.lib", out_dir)))
		.expect("Failed to copy file");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=rustpad_rc");
}