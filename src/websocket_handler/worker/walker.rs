use futures::prelude::Stream;
use jwalk::WalkDir;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileSize(pub PathBuf, pub u64);

pub async fn walk(root_path: PathBuf) -> impl Stream<Item = FileSize> {
  futures::stream::iter(
    WalkDir::new(&root_path)
      .preload_metadata(true)
      .skip_hidden(false)
      .into_iter()
      .filter_map(move |maybe_entry| {
        let entry = maybe_entry.ok()?;

        if entry.file_type.as_ref().ok()?.is_file() {
          let path = entry.path().strip_prefix(&root_path).ok()?.to_path_buf();
          let metadata = entry.metadata?.ok()?;
          let len = metadata.len();

          Some(FileSize(path, len))
        } else {
          None
        }
      }),
  )
}
