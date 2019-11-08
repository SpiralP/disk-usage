mod dir;
mod tree;
mod walker;

pub use self::{dir::*, tree::*, walker::*};
use super::EventMessage;
use futures::{
  channel::mpsc::{UnboundedReceiver, UnboundedSender},
  future::Either,
  prelude::*,
  stream,
};
use log::*;
use std::{collections::HashSet, time::Instant};

async fn send_directory_change(
  root_path: &[String],
  path: &[String],
  tree: &mut Directory,
  event_sender: &mut UnboundedSender<EventMessage>,
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
    .await
    .unwrap();

  dirs
}

pub enum ScannerControlMessage {
  ChangeDirectory(Vec<String>),
}

pub async fn start_scanner_thread(
  root_path: Vec<String>,
  mut control_receiver: UnboundedReceiver<ScannerControlMessage>,
  mut event_sender: UnboundedSender<EventMessage>,
) {
  // use a separate thread for this future because
  // iterator-streams block the tokio threadpool
  tokio::spawn(async move {
    let mut tree = Directory::new();

    let start_time = Instant::now();

    let mut current_dir;
    let mut subscribed_dirs;

    // wait for default current directory
    match control_receiver.next().await.unwrap() {
      ScannerControlMessage::ChangeDirectory(path) => {
        subscribed_dirs =
          send_directory_change(&root_path, &path, &mut tree, &mut event_sender).await;
        current_dir = path;
      }
    }

    let file_size_stream = walk(root_path.iter().collect()).await;

    let mut either_stream = stream::select(
      control_receiver
        .map(Some)
        .chain(stream::once(future::ready(None)))
        .map(Either::Left),
      file_size_stream
        .map(Some)
        .chain(stream::once(future::ready(None)))
        .map(Either::Right),
    );

    while let Some(either) = either_stream.next().await {
      match either {
        Either::Left(control_message) => {
          if let Some(ScannerControlMessage::ChangeDirectory(path)) = control_message {
            subscribed_dirs =
              send_directory_change(&root_path, &path, &mut tree, &mut event_sender).await;
            current_dir = path;
          } else {
            println!("DISCONNECT?");
            break;
          }
        }

        Either::Right(file_size_message) => {
          if let Some(FileSize(path, size)) = file_size_message {
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
              .unbounded_send(EventMessage::SizeUpdate {
                entry: Entry::Directory {
                  name: relative_name.clone(),
                  size,
                },
              })
              .expect("event_sender.send");
          } else {
            let end_time = Instant::now();
            info!("scanner done! {:?}", end_time - start_time);
          }
        }
      }
    } // while either_stream
  });
}
