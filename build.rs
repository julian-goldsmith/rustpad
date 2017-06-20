use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").ok().expect("can't find out_dir");

    Command::new("rc").arg("/r")
					  .arg("/i")
					  .arg("C:\\Program Files (x86)\\Windows Kits\\8.1\\Include\\um")
					  .arg("/i")
					  .arg("C:\\Program Files (x86)\\Windows Kits\\8.1\\Include\\shared")
					  .arg("/i")
					  .arg("D:\\Programs\\Microsoft Visual Studio 12.0\\VC\\include")
					  .arg("/fo")
					  .arg(&format!("{}\\rustpad.res", out_dir))
					  .arg("src\\rustpad.rc")
                      .status().unwrap();
    Command::new("lib")
					  .arg(&format!("/out:rustpad_rc.lib"))
					  .arg(&format!("rustpad.res"))
					  .arg("/machine:x64")
					  .arg("/verbose")
                      .current_dir(&Path::new(&out_dir))
                      .status().unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=rustpad_rc");
}