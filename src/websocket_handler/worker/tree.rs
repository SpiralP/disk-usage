#![allow(dead_code)]

use crate::websocket_handler::{
  api::{Entry, UpdatingStatus},
  worker::walker::{DirStatus, FileSize, FileType},
};
use std::{collections::HashMap, path::Path};

// TODO make get_total_size cache!
// maybe entries_mut sets dirty flag? then all entries_mut will have to be recomputed
// maybe subscribe to a certain folder's events and call a think function to calculate?

#[derive(Debug)]
pub struct Directory {
  pub updating: UpdatingStatus,
  pub total_size: u64,
  entries: HashMap<String, Directory>,
}

impl Default for Directory {
  fn default() -> Self {
    Self::new()
  }
}

impl Directory {
  pub fn new() -> Self {
    Self {
      updating: UpdatingStatus::Idle,
      total_size: 0,
      entries: HashMap::new(),
    }
  }

  pub fn at_mut(&mut self, components: &[String]) -> Option<&mut Self> {
    let mut current = self;
    for component in components {
      current = current.entries.get_mut(component)?;
    }

    Some(current)
  }

  fn set_updating(&mut self, components: &[String], updating: UpdatingStatus) {
    let mut current = self;
    for component in components {
      current = current
        .entries
        .entry(component.clone())
        .or_insert_with(Self::new);
    }

    current.updating = updating;
  }

  fn add_file(&mut self, components: &[String], size: u64) {
    // <root>/hello/world/

    let mut current = self;
    // root tree total_size += size
    current.total_size += size;

    // update 'hello' then 'world'
    for component in components {
      // this unwrap is ok because we do set_updating ALWAYS before add_file
      current = current.entries.get_mut(component).unwrap();

      current.total_size += size;
    }
  }

  pub fn update(&mut self, file_type: &FileType) {
    match file_type {
      FileType::Dir(path, status) => {
        let components = get_components(&path);
        let updating = if let DirStatus::Started = status {
          UpdatingStatus::Updating
        } else {
          UpdatingStatus::Finished
        };

        self.set_updating(&components, updating);
      }

      FileType::File(FileSize(path, size)) => {
        let components = get_components(&path);
        // remove filename
        self.add_file(&components[..components.len() - 1], *size);
      }
    }
  }

  pub fn get_entry_directory(&mut self, path: Vec<String>) -> Entry {
    let (size, updating) = self
      .at_mut(&path)
      .map_or((0, UpdatingStatus::Idle), |entry| {
        (entry.total_size, entry.updating)
      });

    Entry::Directory {
      path,
      size,
      updating,
    }
  }
}

pub fn get_components<B: AsRef<Path>>(path: B) -> Vec<String> {
  path
    .as_ref()
    .iter()
    .map(|os_str| os_str.to_string_lossy().to_string())
    .collect()
}

#[ignore]
#[test]
fn test_tree() {
  use super::walker::walk;

  crate::logger::initialize(true, false);

  let mut t = Directory::new();

  let file_size_stream = walk("src".parse().unwrap());
  for file_type in file_size_stream {
    t.update(&file_type);
    println!("{:?} {:#?}", file_type, t);
  }
}
