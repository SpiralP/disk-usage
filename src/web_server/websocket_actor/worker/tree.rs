#![allow(dead_code)]

use std::{collections::HashMap, path::PathBuf};

// TODO make get_total_size cache!
// maybe entries_mut sets dirty flag? then all entries_mut will have to be recomputed
// maybe subscribe to a certain folder's events and call a think function to calculate?

#[derive(Debug)]
pub struct Directory {
  pub total_size: u64,
  entries: HashMap<String, Directory>,
}

impl Directory {
  pub fn new() -> Self {
    Self {
      total_size: 0,
      entries: HashMap::new(),
    }
  }

  fn at(&self, components: Vec<String>) -> Option<&Directory> {
    let mut current = self;
    for component in components {
      current = current.entries.get(&component)?;
    }

    Some(current)
  }

  fn at_path(&self, path: PathBuf) -> Option<&Directory> {
    self.at(get_components(&path))
  }

  pub fn at_mut(&mut self, components: Vec<String>) -> Option<&mut Self> {
    let mut current = self;
    for component in components {
      current = current.entries.get_mut(&component)?;
    }

    Some(current)
  }

  pub fn insert_file(&mut self, components: &[String], size: u64) {
    let mut current = self;
    current.total_size += size;

    // remove filename
    for component in components.iter().take(components.len() - 1) {
      current = current
        .entries
        .entry(component.to_owned())
        .or_insert_with(Directory::new);
      current.total_size += size;
    }
  }
}

#[test]
fn test_tree() {
  use super::{FileSize, FileSizeScanner};

  let (scanner, receiver) = FileSizeScanner::start("src".parse().unwrap());

  let mut t = Directory::new();

  for FileSize(path, size) in receiver {
    let components = get_components(&path);
    t.insert_file(&components, size);
  }

  scanner.join();

  println!("{:#?}", t);
  println!("{}", t.total_size);
}

pub fn get_components(path: &PathBuf) -> Vec<String> {
  path
    .iter()
    .map(|os_str| os_str.to_string_lossy().to_string())
    .collect()
}
