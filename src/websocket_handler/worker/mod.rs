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
use std::{collections::HashSet, thread, time::Instant};

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

#[derive(Debug)]
pub enum ScannerControlMessage {
  ChangeDirectory(Vec<String>),
}

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::too_many_lines)]
pub async fn spawn_scanner_stream(
  root_path: Vec<String>,
  mut control_receiver: UnboundedReceiver<ScannerControlMessage>,
) -> UnboundedReceiver<EventMessage> {
  let (mut event_sender, event_receiver) = unbounded();

  // Use a separate thread for this future because
  // iterator-streams block the tokio threadpool,
  // which causes the control_receiver to never be heard

  thread::spawn(move || {
    futures::executor::block_on(async move {
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

      info!("scanning {:?}", root_path);
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
            debug!("control_receiver hangup");
            break;
          }

          Either::Left(Some(ScannerControlMessage::ChangeDirectory(path))) => {
            debug!("control_receiver ChangeDirectory {:?}", path);

            subscribed_dirs =
              send_directory_change(&root_path, &path, &mut tree, &mut event_sender).await;
            current_dir = path;
          }

          Either::Right(None) => {
            let end_time = Instant::now();
            info!("scanner done! {:?}", end_time - start_time);
          }

          Either::Right(Some(file_type)) => {
            tree.update(&file_type);

            match file_type {
              FileType::File(FileSize(path, _)) => {
                let components = get_components(&path);

                if components.starts_with(&current_dir) {
                  // file is in current directory
                  // This ignores higher directory changes

                  let relative_name = &components[current_dir.len()];
                  if subscribed_dirs.contains(relative_name) {
                    // File is in a directory in the current dir.
                    // Update that directory's size.
                    // This ignores file updates in the current dir

                    let size = tree
                      .at_mut(&components[..=current_dir.len()])
                      .expect("tree.at")
                      .total_size;

                    if let Err(e) = event_sender
                      .send(EventMessage::SizeUpdate {
                        entry: Entry::Directory {
                          name: relative_name.clone(),
                          size,
                          updating: true,
                        },
                      })
                      .await
                    {
                      warn!("scanner to event_sender: {}", e);
                    }
                  }
                }
              }

              FileType::Dir(path, status) => {
                let updating = if let DirStatus::Started = status {
                  true
                } else {
                  false
                };

                let components = get_components(&path);

                if components.is_empty() {
                  let size = tree.total_size;

                  if let Err(e) = event_sender
                    .send(EventMessage::SizeUpdate {
                      entry: Entry::Directory {
                        name: String::new(),
                        size,
                        updating,
                      },
                    })
                    .await
                  {
                    warn!("scanner to event_sender: {}", e);
                  }
                } else if components.starts_with(&current_dir)
                  && components.len() == current_dir.len() + 1
                {
                  // Dir is in our current dir
                  // We don't care about recursion
                  let relative_name = &components[current_dir.len()];
                  let size = tree.at_mut(&components).map_or(0, |dir| dir.total_size);

                  if let Err(e) = event_sender
                    .send(EventMessage::SizeUpdate {
                      entry: Entry::Directory {
                        name: relative_name.clone(),
                        size,
                        updating,
                      },
                    })
                    .await
                  {
                    warn!("scanner to event_sender: {}", e);
                  }
                }
              }
            }
          }
        }
      } // while either_stream

      debug!("scanner either_stream completed");
    });
  });

  event_receiver
}
