use super::{get_components, Directory};
use serde::Serialize;
use std::{fs, path::*};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Entry {
  File { name: String, size: u64 },
  Directory { name: String, size: u64 },
}

pub fn get_directory_entries(
  root_path: &[String],
  path: &[String],
  tree: &mut Directory,
) -> Vec<Entry> {
  // root_path: ["src"]
  // path: ["web_server", "websocket_actor"]


  // src/web_server/websocket_actor
  let full_path: PathBuf = root_path.iter().chain(path).collect();
  let root_path: PathBuf = root_path.iter().collect();

  fs::read_dir(full_path)
    .expect("read_dir")
    .map(move |maybe_entry| {
      let entry = maybe_entry.expect("maybe_entry");

      let path = entry.path();
      let name = path
        .file_name()
        .expect("file_name")
        .to_string_lossy()
        .to_string();
      let file_type = entry.file_type().expect("file_type");

      if file_type.is_dir() {
        let relative_path = get_components(
          path
            .strip_prefix(&root_path)
            .expect("strip_prefix")
            .to_owned(),
        );

        let size = tree.at_mut(&relative_path).map_or(0, |dir| dir.total_size);

        Entry::Directory { name, size }
      } else {
        let metadata = entry.metadata().expect("metadata");
        let size = metadata.len();

        Entry::File { name, size }
      }
    })
    .collect()
}

// #[test]
// fn test_get_directory_entries() {
//   use super::FileSize;

//   let mut tree = Tree::new();

//   tree.insert_file(FileSize("hello.rs".parse().unwrap(), 123));
//   tree.insert_file(FileSize("web_server/file.rs".parse().unwrap(), 123));

//   println!(
//     "{:#?}",
//     get_directory_entries(&"src".parse().unwrap(), &tree)
//   );
// }
