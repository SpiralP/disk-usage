mod dir;
mod tree;

pub use self::{dir::*, tree::*};
use super::EventMessage;
use crossbeam::channel::{select, Receiver, Sender};
use directory_size::*;
use std::thread;

fn send_directory_change(
  root_path: &[String],
  path: Vec<String>,
  tree: &Tree,
  event_sender: &Sender<EventMessage>,
) -> (Vec<String>, Vec<String>) {
  let entries = get_directory_entries(root_path, path.clone(), &tree);

  event_sender
    .send(EventMessage::DirectoryChange {
      path: path.clone(),
      entries: entries.clone(),
    })
    .unwrap();

  (
    path,
    entries
      .iter()
      .filter_map(move |entry| {
        if let Entry::Directory { name, .. } = entry {
          Some(name.to_owned())
        } else {
          None
        }
      })
      .collect(),
  )
}

pub enum ThreadControlMessage {
  ChangeDirectory(Vec<String>),
}

pub fn start_scanner_thread(
  root_path: Vec<String>,
  thread_control_receiver: Receiver<ThreadControlMessage>,
  event_sender: Sender<EventMessage>,
) -> thread::JoinHandle<()> {
  thread::spawn(move || {
    let (_scanner, file_receiver) = FileSizeScanner::start(root_path.iter().collect());

    let mut tree = Tree::new();

    // wait for default current directory
    let (mut current_dir, mut subscribed_entries): (Vec<String>, Vec<String>) =
      match thread_control_receiver.recv().unwrap() {
        ThreadControlMessage::ChangeDirectory(path) => {
          send_directory_change(&root_path, path, &tree, &event_sender)
        }
      };

    loop {
      select! {
        recv(thread_control_receiver) -> control_message => {
          let control_message = control_message.unwrap();

          match control_message {
            ThreadControlMessage::ChangeDirectory(path) => {
              println!("thread got dir change! {:?}", path);

              let (a, b) = send_directory_change(&root_path, path, &tree, &event_sender);
              current_dir = a;
              subscribed_entries = b;
            }
          }
        },

        recv(file_receiver) -> file => {
          let file = match file {
            Ok(send_directory_change) => send_directory_change,
            Err(_) => {
              break;
            }
          };

          let components = get_components(&file.0);
          tree.insert_file(file);

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
          let size = tree.at(bap).unwrap().get_total_size();

          event_sender.send(EventMessage::SizeUpdate {
            entry: Entry::Directory {
              name: relative_name,
              size,
            }
          }).unwrap();
        },
      }
    }

    for control_message in thread_control_receiver {
      match control_message {
        ThreadControlMessage::ChangeDirectory(path) => {
          println!("thread got dir change (after live update)! {:?}", path);

          send_directory_change(&root_path, path, &tree, &event_sender);
        }
      }
    }
  })
}
