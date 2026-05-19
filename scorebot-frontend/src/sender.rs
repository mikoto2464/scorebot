use anyhow::Context;
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
    pub fn new() -> anyhow::Result<Self> {
        let (tx, mut rx) = mpsc::channel::<Message>(1000);
        let (ctrl_tx, ctrl_rx) = watch::channel(true);

        let sender = Self { tx, ctrl_tx };

        sender.spawn_worker(rx, ctrl_rx)?;

        Ok(sender)
    }

    fn spawn_worker(&self, mut rx: mpsc::Receiver<Message>, mut ctrl_rx: watch::Receiver<bool>) -> anyhow::Result<()> {
        tokio::task::spawn(async move {
            let mut active_stream: Option<LocalSocketStream> = None;
            loop {
                let is_running = *ctrl_rx.borrow();

                if !is_running {
                    if active_stream.is_some() {
                        info!("Stream Closed");
                        active_stream = None;
                    }

                    if ctrl_rx.changed().await.is_err() {
                        break;
                    }
                    continue;
                }

                if active_stream.is_none() {
                    let socket_name = SOCKET_NAME.to_fs_name_if_supported()?;

                    active_stream = Some(LocalSocketStream::connect(socket_name).await
                        .context("Could not connect to socket")?);
                }

                select! {
                    maybe_msg = rx.recv() => {
                        if let Some(msg) = maybe_msg {
                            if let Some(ref mut stream) = active_stream {
                                let payload = rkyv::to_bytes::<Message>(&msg)?;
                                stream.write_all(&payload).await?;
                            } else {
                                break;
                            }
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