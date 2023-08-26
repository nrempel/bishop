use std::net::SocketAddr;
use thiserror::Error;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

use crate::codec::{Codec, Message};

#[derive(Debug)]
pub struct Daemon {
    addr: SocketAddr,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to bind to address: {0}")]
    BindError(#[from] std::io::Error),

    #[error("error sending message: {0}")]
    SendMessageError(#[from] mpsc::error::SendError<std::io::Error>),
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
                if let Ok(message) = result {
                    Self::process_message(message);
                } else if let Err(err) = result {
                    eprintln!("error decoding message: {:?}", err);
                }
            }
        });

        while let Ok((socket, _)) = listener.accept().await {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut reader = FramedRead::new(socket, Codec);
                while let Some(message_result) = reader.next().await {
                    if let Err(err) = tx.send(message_result) {
                        eprintln!("error sending message: {:?}", err);
                    }
                }
            });
        }

        Ok(())
    }

    fn process_message(message: Message) {
        match message {
            Message::Ping => {
                println!("Ping message received");
                if let Some(response) = message.response() {
                    match response {
                        Message::Pong => {
                            println!("Sending Pong response");
                            // Code to write the Pong response
                        }
                        _ => {
                            eprintln!("Unexpected response to Message::Ping");
                        }
                    }
                }
            }
            Message::Pong => eprintln!("Not expecting Pong message"),
        }
    }
}
