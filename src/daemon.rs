use std::net::SocketAddr;

use thiserror::Error;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

use crate::codec::Codec;

#[derive(Debug)]
pub struct Daemon {
    addr: SocketAddr,
}

impl Daemon {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn run(self) -> Result<(), Error> {
        let listener = TcpListener::bind(&self.addr).await?;
        let (tx, mut rx) = mpsc::unbounded_channel();

        println!("Listening on: {}", self.addr);

        tokio::spawn(async move {
            while let Some(result) = rx.recv().await {
                // TODO: handle messages
                match result {
                    Ok(message) => println!("received: {message:?}"),
                    Err(err) => eprintln!("error decoding message: {err}"),
                }
            }
        });

        while let Ok((socket, _)) = listener.accept().await {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut reader = FramedRead::new(socket, Codec);
                while let Some(message_result) = reader.next().await {
                    if let Err(err) = tx.send(message_result) {
                        eprintln!("error sending message: {err}");
                    }
                }
            });
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to bind to address: {0}")]
    BindError(#[from] std::io::Error),

    #[error("error sending message: {0}")]
    SendMessageError(#[from] mpsc::error::SendError<std::io::Error>),
}
