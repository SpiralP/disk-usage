use jwalk::WalkDir;
use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct FileSize(pub PathBuf, pub u64);

#[derive(Debug, Clone)]
pub enum DirStatus {
  Started,
  Finished,
}

#[derive(Debug, Clone)]
pub enum FileType {
  File(FileSize),
  Dir(PathBuf, DirStatus),
}

pub fn walk(root_path: PathBuf) -> impl Iterator<Item = FileType> {
  // Rc<RefCell<>> causes a compiler panic :(
  let current_dirs = Arc::new(Mutex::new(Vec::new()));
  let current_dirs2 = current_dirs.clone();

  WalkDir::new(&root_path)
    .preload_metadata(true)
    .skip_hidden(false)
    .sort(false)
    .into_iter()
    .filter_map(move |maybe_entry| {
      let mut current_dirs = current_dirs.lock().unwrap();

      let entry = maybe_entry.ok()?;

      let file_type = entry.file_type.as_ref().ok()?;
      let path = entry.path().strip_prefix(&root_path).ok()?.to_path_buf();

      let mut out = Vec::new();

      while let Some((dir, parent)) = current_dirs
        .last()
        .and_then(|dir| Some((dir, path.parent()?)))
      {
        if parent == dir {
          break;
        } else {
          out.push(FileType::Dir(
            current_dirs.pop().unwrap(),
            DirStatus::Finished,
          ));
        }
      }

      if file_type.is_dir() {
        current_dirs.push(path.clone());
        out.push(FileType::Dir(path, DirStatus::Started))
      } else if file_type.is_file() {
        let metadata = entry.metadata?.ok()?;
        let len = metadata.len();

        out.push(FileType::File(FileSize(path, len)))
      } else {
        // None
      }

      Some(out)
    })
    .flatten()
    .chain(std::iter::from_fn(move || {
      let mut current_dirs = current_dirs2.lock().unwrap();
      current_dirs
        .pop()
        .map(move |dir| FileType::Dir(dir, DirStatus::Finished))
    }))
}

#[test]
fn test_walk() {
  for item in walk("test-folder".parse().unwrap()) {
    if let FileType::Dir(..) = item {
      println!("{:?}", item);
    }
  }
}
