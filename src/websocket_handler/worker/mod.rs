mod dir;
mod tree;
mod walker;

pub use self::{dir::*, tree::*, walker::*};
use super::api::{Entry, EventMessage};
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    future::Either,
    prelude::*,
    stream,
};
use log::{debug, info, warn};
use std::{collections::HashSet, thread, time::Instant};

// returns "subscribed dirs"
// which are ones in current dir
// and those leading up to current path for breadcrumb updates
async fn send_directory_change(
    root_path: &[String],
    path: &[String],
    tree: &mut Directory,
    event_sender: &mut UnboundedSender<EventMessage>,
) -> HashSet<Vec<String>> {
    let (entries, available_space) = get_directory_entries(root_path, path, tree);

    let mut subscribed_dirs: HashSet<Vec<String>> = entries
        .iter()
        .filter_map(move |entry| {
            if let Entry::Directory { path, .. } = entry {
                Some(path.clone())
            } else {
                None
            }
        })
        .collect();

    let mut cur = Vec::new();
    let mut breadcrumb_entries = Vec::new();

    // root tree
    subscribed_dirs.insert(cur.to_vec());
    breadcrumb_entries.push(tree.get_entry_directory(cur.to_vec()));

    for component in path {
        cur.push(component.to_string());
        subscribed_dirs.insert(cur.to_vec());
        breadcrumb_entries.push(tree.get_entry_directory(cur.to_vec()));
    }

    if let Err(e) = event_sender
        .send(EventMessage::DirectoryChange {
            current_directory: tree.get_entry_directory(path.to_vec()),
            entries,
            breadcrumb_entries,
            available_space,
        })
        .await
    {
        warn!("send_directory_change: {}", e);
    }

    subscribed_dirs
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
                        send_directory_change(&root_path, &path, &mut tree, &mut event_sender)
                            .await;
                    current_dir = path;
                }
            }

            let file_size_stream = stream::iter(walk(root_path.iter().collect()));

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
                            send_directory_change(&root_path, &path, &mut tree, &mut event_sender)
                                .await;
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
                                // send update for total size of shown directories

                                // [ (src), websocket_handler, worker, mod.rs ]
                                let components = get_components(&path);

                                // current_dir = src
                                // true
                                if components.starts_with(&current_dir) {
                                    // file is in current directory
                                    // This ignores higher directory changes

                                    let components = components[..=current_dir.len()].to_vec();
                                    // current_dir.len() = 1
                                    // [ ( (src), websocket_handler ), worker, mod.rs ]

                                    // [ websocket_handler, src ]
                                    if subscribed_dirs.contains(&components) {
                                        // File is in a directory in the current dir.
                                        // Update that directory's size.

                                        if let Err(e) = event_sender
                                            .send(EventMessage::SizeUpdate {
                                                entry: tree.get_entry_directory(components),
                                            })
                                            .await
                                        {
                                            warn!("scanner to event_sender: {}", e);
                                        }
                                    }
                                }
                            }

                            FileType::Dir(path, _) => {
                                let components = get_components(&path);
                                if subscribed_dirs.contains(&components) {
                                    // Dir is in our current dir
                                    // We don't care about recursion

                                    if let Err(e) = event_sender
                                        .send(EventMessage::SizeUpdate {
                                            entry: tree.get_entry_directory(components),
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
