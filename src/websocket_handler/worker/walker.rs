use jwalk::WalkDirGeneric;
use log::info;
use std::{
    fs::Metadata,
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
    info!("scanning {:?}", root_path);

    // Rc<RefCell<>> causes a compiler panic :(
    let current_dirs = Arc::new(Mutex::new(Vec::new()));
    let current_dirs2 = current_dirs.clone();

    WalkDirGeneric::<((), Option<Result<Metadata, jwalk::Error>>)>::new(&root_path)
        .skip_hidden(false)
        .sort(false)
        .process_read_dir(|_, dir_entry_results| {
            dir_entry_results.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    dir_entry.client_state = Some(dir_entry.metadata());
                }
            })
        })
        .into_iter()
        .filter_map(move |maybe_entry| {
            let mut current_dirs = current_dirs.lock().unwrap();

            let entry = maybe_entry.ok()?;

            let file_type = entry.file_type;

            // for inputs like "." or ".." or "/"
            // the first entry.path() is "/"
            let path = entry.path();
            let path = if path.to_str().unwrap() == "/" {
                PathBuf::new()
            } else {
                path.strip_prefix(&root_path).unwrap().to_path_buf()
            };

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
                let metadata = entry.client_state.unwrap().ok()?;
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

#[ignore]
#[test]
fn test_walk() {
    crate::logger::initialize(true, false);

    for item in walk("test-folder".parse().unwrap()) {
        if let FileType::Dir(..) = item {
            println!("{:?}", item);
        }
    }
}
