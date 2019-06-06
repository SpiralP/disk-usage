use includedir_codegen::Compression;
use std::{fs, process::Command};

fn run(cmd: &str) -> bool {
  if cfg!(target_os = "windows") {
    Command::new("cmd")
      .args(&["/C", cmd])
      .status()
      .unwrap()
      .success()
  } else {
    Command::new("sh")
      .arg("-c")
      .arg(cmd)
      .status()
      .unwrap()
      .success()
  }
}

fn main() {
  use std::env;

  if fs::metadata("node_modules").is_err() {
    assert!(run("yarn install"));
  }

  if !env::var("OUT_DIR").unwrap().contains("/rls/") {
    let _ = fs::remove_dir_all("dist");

    assert!(run("yarn build"));
  }

  includedir_codegen::start("WEB_FILES")
    .dir("dist", Compression::Gzip)
    .build("web_files.rs")
    .unwrap();
}
