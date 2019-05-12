use serde::Serialize;
use std::{fs, path::*};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Entry {
  File { name: String, size: u64 },
  Directory { name: String },
}

pub fn get_directory_entries(root_path: &PathBuf) -> Vec<Entry> {
  fs::read_dir(root_path)
    .unwrap()
    .map(|maybe_entry| {
      let entry = maybe_entry.unwrap();

      let path = entry.path();
      let file_name = path.file_name().unwrap().to_string_lossy().to_string();
      let file_type = entry.file_type().unwrap();
      let metadata = entry.metadata().unwrap();

      if file_type.is_dir() {
        Entry::Directory { name: file_name }
      } else {
        Entry::File {
          name: file_name,
          size: metadata.len(),
        }
      }
    })
    .collect()
}
