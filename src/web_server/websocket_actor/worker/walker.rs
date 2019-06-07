use jwalk::WalkDir;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileSize(pub PathBuf, pub u64);

macro_rules! try_filter {
  ($name:expr) => {
    match $name {
      Ok(v) => v,
      Err(_) => return None,
    }
  };
}

pub fn walk(root_path: PathBuf) -> impl Iterator<Item = FileSize> {
  WalkDir::new(&root_path)
    .preload_metadata(true)
    .skip_hidden(false)
    .into_iter()
    .filter_map(move |maybe_entry| {
      let entry = try_filter!(maybe_entry);

      if try_filter!(entry.file_type).is_file() {
        let path = try_filter!(entry.path().strip_prefix(&root_path)).to_path_buf();
        let metadata = try_filter!(try_filter!(entry.metadata.ok_or(())));
        let len = metadata.len();

        Some(FileSize(path, len))
      } else {
        None
      }
    })
}
