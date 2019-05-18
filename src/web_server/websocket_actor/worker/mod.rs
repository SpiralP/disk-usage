mod dir;
mod tree;

pub use self::{dir::*, tree::*};
use super::EventMessage;
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use directory_size::*;
use log::*;
use std::{collections::HashSet, thread, time::Instant};

fn send_directory_change(
  root_path: &[String],
  path: &[String],
  tree: &mut Directory,
  event_sender: &Sender<EventMessage>,
) -> HashSet<String> {
  let entries = get_directory_entries(root_path, path, tree);

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
      let start_time = Instant::now();
      let (_scanner, file_receiver) = FileSizeScanner::start(root_path.iter().collect());

      let mut tree = Directory::new();

      // wait for default current directory
      let mut current_dir;
      let mut subscribed_entries;

      match control_receiver.recv().unwrap() {
        ThreadControlMessage::ChangeDirectory(path) => {
          subscribed_entries = send_directory_change(&root_path, &path, &mut tree, &event_sender);
          current_dir = path;
        }
      };

      for FileSize(path, size) in file_receiver {
        match control_receiver.try_recv() {
          Ok(control_message) => match control_message {
            ThreadControlMessage::ChangeDirectory(path) => {
              println!("thread got dir change! {:?}", path);

              subscribed_entries =
                send_directory_change(&root_path, &path, &mut tree, &event_sender);
              current_dir = path;
            }
          },

          Err(TryRecvError::Empty) => {}

          Err(TryRecvError::Disconnected) => {
            warn!("control_message disconnected?");
            break;
          }
        }

        let components = get_components(&path);
        tree.insert_file(&components, size);

        if !components.starts_with(&current_dir) {
          // new file is not in current directory
          continue;
        }

        let relative_name = components[current_dir.len()].clone();
        if !subscribed_entries.contains(&relative_name) {
          continue;
        }

        let mut bap = current_dir.clone();
        bap.push(relative_name.clone());

        let size = tree.at_mut(bap).expect("tree.at").total_size;
        event_sender
          .send(EventMessage::SizeUpdate {
            entry: Entry::Directory {
              name: relative_name,
              size,
            },
          })
          .expect("event_sender.send");
      }


      let end_time = Instant::now();
      info!("scanner done! {:?}", end_time - start_time);


      for control_message in control_receiver {
        match control_message {
          ThreadControlMessage::ChangeDirectory(path) => {
            println!("thread got dir change (after live update)! {:?}", path);

            send_directory_change(&root_path, &path, &mut tree, &event_sender);
          }
        }
      }
    })
    .unwrap()
}
