use super::FileSize;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub enum Tree {
  File(u64), // file size
  Directory(HashMap<String, Tree>),
}

impl Tree {
  pub fn new() -> Self {
    Tree::Directory(HashMap::new())
  }

  pub fn entries(&self) -> &HashMap<String, Tree> {
    match self {
      Tree::Directory(map) => map,
      _ => unreachable!("entries"),
    }
  }

  pub fn entries_mut(&mut self) -> &mut HashMap<String, Tree> {
    match self {
      Tree::Directory(map) => map,
      _ => unreachable!("entries_mut"),
    }
  }

  pub fn at(&self, components: Vec<String>) -> Option<&Tree> {
    let mut current = self;
    for component in components {
      let entries = current.entries();

      current = entries.get(&component)?;
    }

    Some(current)
  }

  pub fn at_path(&self, path: PathBuf) -> Option<&Tree> {
    self.at(get_components(&path))
  }

  pub fn at_mut(&mut self, components: Vec<String>) -> Option<&mut Tree> {
    let mut current = self;
    for component in components {
      let entries = current.entries_mut();

      current = entries.get_mut(&component)?;
    }

    Some(current)
  }

  pub fn insert_file(&mut self, FileSize(path, size): FileSize) {
    let mut components = get_components(&path);

    let file_name = components.pop().expect("file_name");

    let mut current = self;
    for component in components {
      let entries = current.entries_mut();

      if !entries.contains_key(&component) {
        entries.insert(component.clone(), Tree::new());
      }

      current = entries.get_mut(&component).expect("get_mut");
    }

    current.entries_mut().insert(file_name, Tree::File(size));
  }

  pub fn get_total_size(&self) -> u64 {
    let entries = self.entries();
    let mut size = 0;

    for item in entries.values() {
      let s = match item {
        Tree::File(size) => *size,
        tree => tree.get_total_size(),
      };

      size += s;
    }

    size
  }
}

#[test]
fn test_tree() {
  use super::FileSizeScanner;

  let (scanner, receiver) = FileSizeScanner::start("src".parse().unwrap());

  let mut t = Tree::new();

  for file in receiver {
    t.insert_file(file);
  }

  scanner.join();

  println!("{:#?}", t);
  println!("{}", t.get_total_size());
}

pub fn get_components(path: &PathBuf) -> Vec<String> {
  path
    .iter()
    .map(|os_str| os_str.to_string_lossy().to_string())
    .collect()
}
