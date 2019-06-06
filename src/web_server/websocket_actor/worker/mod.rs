mod dir;
mod tree;
mod walker;

pub use self::{dir::*, tree::*, walker::*};
use super::EventMessage;
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use log::*;
use std::{collections::HashSet, thread, time::Instant};

fn send_directory_change(
  root_path: &[String],
  path: &[String],
  tree: &mut Directory,
  event_sender: &Sender<EventMessage>,
) -> HashSet<String> {
  let (entries, free_space) = get_directory_entries(root_path, path, tree);

  let dirs = entries
    .iter()
    .filter_map(move |entry| {
      if let Entry::Directory { name, .. } = entry {
        Some(name.to_owned())
      } else {
        None
      }
    })
    .collect();

  event_sender
    .send(EventMessage::DirectoryChange {
      path: path.to_owned(),
      entries,
      free: free_space,
    })
    .unwrap();

  dirs
}

pub enum ThreadControlMessage {
  ChangeDirectory(Vec<String>),
}

pub fn start_scanner_thread(
  root_path: Vec<String>,
  control_receiver: Receiver<ThreadControlMessage>,
  event_sender: Sender<EventMessage>,
) -> thread::JoinHandle<()> {
  thread::Builder::new()
    .name("scanner".to_string())
    .spawn(move || {
      let mut tree = Directory::new();

      // live update
      {
        let start_time = Instant::now();

        let mut current_dir;
        let mut subscribed_dirs;

        // wait for default current directory
        match control_receiver.recv().unwrap() {
          ThreadControlMessage::ChangeDirectory(path) => {
            subscribed_dirs = send_directory_change(&root_path, &path, &mut tree, &event_sender);
            current_dir = path;
          }
        };
        for FileSize(path, size) in walk(root_path.iter().collect()) {
          match control_receiver.try_recv() {
            Err(TryRecvError::Empty) => {}

            Err(TryRecvError::Disconnected) => {
              warn!("control_message disconnected?");
              break;
            }

            Ok(control_message) => match control_message {
              ThreadControlMessage::ChangeDirectory(path) => {
                subscribed_dirs =
                  send_directory_change(&root_path, &path, &mut tree, &event_sender);
                current_dir = path;
              }
            },
          }

          let components = get_components(path);
          tree.insert_file(&components, size);

          if !components.starts_with(&current_dir) {
            // new file is not in current directory
            continue;
          }

          let relative_name = &components[current_dir.len()];
          if !subscribed_dirs.contains(relative_name) {
            continue;
          }

          let size = tree
            .at_mut(&components[..=current_dir.len()])
            .expect("tree.at")
            .total_size;
          event_sender
            .send(EventMessage::SizeUpdate {
              entry: Entry::Directory {
                name: relative_name.clone(),
                size,
              },
            })
            .expect("event_sender.send");
        }

        let end_time = Instant::now();
        info!("scanner done! {:?}", end_time - start_time);
      }

      for control_message in control_receiver {
        match control_message {
          ThreadControlMessage::ChangeDirectory(path) => {
            send_directory_change(&root_path, &path, &mut tree, &event_sender);
          }
        }
      }
    })
    .unwrap()
}
