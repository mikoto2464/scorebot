use std::time::Duration;

use interprocess::local_socket::{GenericFilePath, GenericNamespaced, NameType, ToFsName, ToNsName};
use interprocess::local_socket::tokio::prelude::LocalSocketStream;
use interprocess::local_socket::traits::tokio::Stream;
use scorebot_common::{Message, SOCKET_NAME};
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::sync::{mpsc, watch};
use tracing::{error, info};

#[derive(Clone)]
pub struct QueueCacheSender {
    tx: mpsc::Sender<Message>,
    ctrl_tx: watch::Sender<bool>,
}

impl QueueCacheSender {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Message>(1000);
        let (ctrl_tx, ctrl_rx) = watch::channel(true);

        let sender = Self { tx, ctrl_tx };

        sender.spawn_worker(rx, ctrl_rx);

        sender
    }

    fn spawn_worker(&self, mut rx: mpsc::Receiver<Message>, mut ctrl_rx: watch::Receiver<bool>) {
        tokio::task::spawn(async move {
            let mut active_stream: Option<LocalSocketStream> = None;
            loop {
                let is_running = *ctrl_rx.borrow();

                if !is_running {
                    if active_stream.take().is_some() {
                        info!("Stream closed");
                    }

                    if ctrl_rx.changed().await.is_err() {
                        break;
                    }
                    continue;
                }

                if active_stream.is_none() {
                    let socket_name = if GenericNamespaced::is_supported() {
                        SOCKET_NAME.to_ns_name::<GenericNamespaced>()
                    } else {
                        SOCKET_NAME.to_fs_name::<GenericFilePath>()
                    };

                    let socket_name = match socket_name {
                        Ok(name) => name,
                        Err(e) => {
                            error!("Failed to resolve socket name: {e}");
                            break;
                        }
                    };

                    match LocalSocketStream::connect(socket_name).await {
                        Ok(stream) => active_stream = Some(stream),
                        Err(e) => {
                            error!("Failed to connect to socket: {e}");
                            tokio::time::sleep(Duration::from_millis(1000)).await;
                            continue;
                        }
                    }
                }

                select! {
                    maybe_msg = rx.recv() => {
                        let Some(msg) = maybe_msg else {
                            break;
                        };

                        let Some(ref mut stream) = active_stream else {
                            break;
                        };

                        let payload = match rkyv::to_bytes::<rkyv::rancor::Error>(&msg) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                error!("Failed to serialize message: {e}");
                                continue;
                            }
                        };

                        if let Err(e) = stream.write_all(&payload).await {
                            error!("Failed to write to socket: {e}");
                            active_stream = None;
                        }
                    }
                    _ = ctrl_rx.changed() => { continue; }
                }
            }
            info!("Worker exited");
        });
    }

    pub async fn send(&self, message: Message) {
        self.tx.send(message).await.unwrap();
    }
}
