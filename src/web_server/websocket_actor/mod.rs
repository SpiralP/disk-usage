mod binary_message;
mod dir;
mod text_message;
mod tree;

use self::{binary_message::*, dir::*, text_message::*, tree::*};
use actix::prelude::*;
use actix_web_actors::ws;
use crossbeam::channel::{self, select, Sender, TryRecvError};
use directory_size::*;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{path::PathBuf, thread, time::Duration};

pub struct WebSocketActor {
  root_path: Vec<String>,
  current_dir: Vec<String>,
  thread_sender: Option<Sender<ThreadControlMessage>>,
  scanner: Option<FileSizeScanner>,
}

impl WebSocketActor {
  pub fn new(root_path: &PathBuf) -> Self {
    Self {
      root_path: get_components(root_path),
      current_dir: Vec::new(),
      thread_sender: None,
      scanner: None,
    }
  }

  fn change_dir(&mut self, path: Vec<String>) {
    self.current_dir = path.clone();

    self
      .thread_sender
      .as_ref()
      .unwrap()
      .send(ThreadControlMessage::ChangeDirectory(path))
      .unwrap();
  }
}

fn send_directory_change(
  root_path: &[String],
  path: Vec<String>,
  tree: &Tree,
  addr: &Addr<WebSocketActor>,
) -> (Vec<String>, Vec<String>) {
  let entries = get_directory_entries(root_path, path.clone(), &tree);
  addr.do_send(TextMessage(
    serde_json::to_string(&EventMessage::DirectoryChange {
      path: path.clone(),
      entries: entries.clone(),
    })
    .unwrap(),
  ));

  (
    path,
    entries
      .iter()
      .filter_map(|entry| {
        if let Entry::Directory { name, .. } = entry {
          Some(name.to_owned())
        } else {
          None
        }
      })
      .collect(),
  )
}


enum ThreadControlMessage {
  ChangeDirectory(Vec<String>),
}

impl Actor for WebSocketActor {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    info!("ws started");

    let (thread_sender, thread_receiver) = channel::unbounded();
    self.thread_sender = Some(thread_sender);

    // send initial current directory entries
    self.change_dir(Vec::new());


    let _thread = {
      let addr = ctx.address();
      let root_path: Vec<String> = self.root_path.clone();

      let (scanner, file_receiver) = FileSizeScanner::start(root_path.iter().collect());
      self.scanner = Some(scanner);

      thread::spawn(move || {
        let mut tree = Tree::new();

        // wait for default current directory
        let (mut current_dir, mut subscribed_entries): (Vec<String>, Vec<String>) =
          match thread_receiver.recv().unwrap() {
            ThreadControlMessage::ChangeDirectory(path) => {
              send_directory_change(&root_path, path, &tree, &addr)
            }
          };

        loop {
          select! {
            recv(thread_receiver) -> control_message => {
              let control_message = control_message.unwrap();

              match control_message {
                ThreadControlMessage::ChangeDirectory(path) => {
                  println!("thread got dir change! {:?}", path);

                  let (a, b) = send_directory_change(&root_path, path, &tree, &addr);
                  current_dir = a;
                  subscribed_entries = b;
                }

                _ => unreachable!("control_message")
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

              addr.do_send(TextMessage(
                serde_json::to_string(&EventMessage::SizeUpdate {
                  entry: Entry::Directory {
                    name: relative_name,
                    size,
                  }
                })
                .unwrap(),
              ));
            },
          }
        }


        for control_message in thread_receiver {
          match control_message {
            ThreadControlMessage::ChangeDirectory(path) => {
              println!("thread got dir change (after live update)! {:?}", path);

              send_directory_change(&root_path, path, &tree, &addr);
            }

            _ => unreachable!("control_message"),
          }
        }
      })
    };

    // thread::spawn(move || {
    //   thread::sleep(Duration::from_millis(512));

    //   loop {
    //     // send chunk
    //     let mut chunk = Vec::new();
    //     let mut finished = false;
    //     let mut take_a_break = false;
    //     for _ in 0..10240 {
    //       // limit chunk size because it's hella big and makes chrome lag

    //       match receiver.try_recv() {
    //         Ok(file) => {
    //           chunk.push(FileSizeJson::from(file));
    //         }

    //         Err(e) => match e {
    //           TryRecvError::Empty => {
    //             take_a_break = true;
    //           }
    //           _ => {
    //             finished = true;
    //             break;
    //           }
    //         },
    //       }
    //     }

    //     info!("chunk with {}", chunk.len());
    //     addr.do_send(TextMessage(
    //       serde_json::to_string(&FileSizeStatusJson::Chunk(chunk)).unwrap(),
    //     ));

    //     if finished {
    //       break;
    //     }

    //     if take_a_break {
    //       thread::sleep(Duration::from_millis(512));
    //     }
    //   } // loop

    //   scanner.join();

    //   // send finish
    //   addr.do_send(TextMessage(
    //     serde_json::to_string(&FileSizeStatusJson::Finish).unwrap(),
    //   ));
    // });
  }

  fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
    info!("ws stopping");

    Running::Stop
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {
    info!("ws stopped");
  }
}


impl Handler<TextMessage> for WebSocketActor {
  type Result = ();

  fn handle(&mut self, msg: TextMessage, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(msg.0);
  }
}

impl Handler<BinaryMessage> for WebSocketActor {
  type Result = ();

  fn handle(&mut self, msg: BinaryMessage, ctx: &mut Self::Context) -> Self::Result {
    ctx.binary(msg.0);
  }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocketActor {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    match msg {
      ws::Message::Text(text) => {
        let control_message: ControlMessage = serde_json::from_str(&text).unwrap();
        info!("{:#?}", control_message);

        match control_message {
          ControlMessage::ChangeDirectory { path } => {
            self.change_dir(path);
          }
        }
      }

      ws::Message::Close(_) => ctx.stop(),

      _ => {
        warn!("other ws kind! {:#?}", msg);
      }
    }
  }
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum EventMessage {
  DirectoryChange {
    path: Vec<String>,
    entries: Vec<Entry>,
  },

  SizeUpdate {
    entry: Entry,
  },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum ControlMessage {
  ChangeDirectory { path: Vec<String> },
}
