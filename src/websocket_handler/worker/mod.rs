mod dir;
mod tree;
mod walker;

pub use self::{dir::*, tree::*, walker::*};
use super::EventMessage;
use futures::{
  channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
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

  if let Err(e) = event_sender
    .send(EventMessage::DirectoryChange {
      path: path.to_owned(),
      entries,
      free: free_space,
    })
    .await
  {
    warn!("send_directory_change: {}", e);
  }

  dirs
}

pub enum ScannerControlMessage {
  ChangeDirectory(Vec<String>),
}

pub async fn spawn_scanner_stream(
  root_path: Vec<String>,
  mut control_receiver: UnboundedReceiver<ScannerControlMessage>,
) -> UnboundedReceiver<EventMessage> {
  let (mut event_sender, event_receiver) = unbounded();

  // TODO is this still true??
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

    let file_size_stream = futures::stream::iter(walk(root_path.iter().collect()));

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
        Either::Left(None) => {
          break;
        }

        Either::Left(Some(control_message)) => match control_message {
          ScannerControlMessage::ChangeDirectory(path) => {
            subscribed_dirs =
              send_directory_change(&root_path, &path, &mut tree, &mut event_sender).await;
            current_dir = path;
          }
        },

        Either::Right(None) => {
          let end_time = Instant::now();
          info!("scanner done! {:?}", end_time - start_time);
        }

        Either::Right(Some(file_type)) => {
          match file_type {
            FileType::File(FileSize(path, size)) => {
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

              if let Err(e) = event_sender
                .send(EventMessage::SizeUpdate {
                  entry: Entry::Directory {
                    name: relative_name.clone(),
                    size,
                  },
                })
                .await
              {
                warn!("scanner to event_sender: {}", e);
              }
            }

            FileType::Dir(path, status) => {
              println!("{:?} {:?}", status, path);
            }
          }
        }
      }
    } // while either_stream

    debug!("scanner either_stream completed");
  });

  event_receiver
}
