use includedir_codegen::Compression;
use std::process::Command;

fn main() {
  use std::env;

  if !env::var("OUT_DIR").unwrap().contains("/rls/") {
    let ok = if cfg!(target_os = "windows") {
      Command::new("cmd")
        .args(&["/C", "yarn build"])
        .status()
        .unwrap()
        .success()
    } else {
      Command::new("sh")
        .arg("-c")
        .arg("yarn build")
        .status()
        .unwrap()
        .success()
    };
    assert!(ok);
  }


  includedir_codegen::start("WEB_FILES")
    .dir("dist", Compression::Gzip)
    .build("web_files.rs")
    .unwrap();
}
