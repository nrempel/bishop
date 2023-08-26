use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

use bishop::codec::{Codec, Message};
use bishop::daemon::Daemon;
use clap::Parser;
use daemonize::Daemonize;

use futures_util::SinkExt;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

fn main() -> Result<(), Error> {
    let args = Args::parse();
    match args.cmd {
        Commands::Up(args) => up(args),
        Commands::Ping(args) => ping(args),
    }
}

fn up(args: Up) -> Result<(), Error> {
    let daemon = Daemon::new(args.addr);
    fs::create_dir_all(args.pid_file.parent().ok_or(Error::NoParentDir)?)?;

    let daemonize = Daemonize::new()
        .pid_file(args.pid_file)
        .privileged_action(move || {
            let rt = Runtime::new()?;
            rt.block_on(async { daemon.run().await })
        });

    let result = daemonize.start()?;
    println!("Daemonize result: {:?}", result);

    Ok(())
}

fn ping(args: Ping) -> Result<(), Error> {
    let rt = Runtime::new()?;
    let result = rt.block_on(async {
        let stream = TcpStream::connect(&args.addr).await?;
        let (reader, writer) = stream.into_split();
        let mut reader = FramedRead::new(reader, Codec);
        let mut writer = FramedWrite::new(writer, Codec);
        writer.send(Message::Ping).await?;
        match reader.next().await {
            Some(Ok(Message::Pong)) => {
                println!("Received Pong");
                Ok(())
            }
            Some(Ok(_)) => Err(Error::UnexpectedMessage),
            Some(Err(err)) => Err(err.into()),
            None => Err(Error::UnexpectedEOF),
        }
    });

    result
}

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(Parser)]
enum Commands {
    Up(Up),
    Ping(Ping),
}

#[derive(Parser)]
struct Up {
    #[clap(long, default_value = "/tmp/bishop.pid")]
    pid_file: PathBuf,
    #[clap(long, default_value = "127.0.0.1:8765")]
    addr: SocketAddr,
}

#[derive(Parser)]
struct Ping {
    #[clap(long, default_value = "127.0.0.1:8765")]
    addr: SocketAddr,
}

#[derive(Debug, Error)]
enum Error {
    #[error("failed to create pid file parent directory")]
    NoParentDir,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to start daemon: {0}")]
    Daemonize(#[from] daemonize::Error),
    #[error("tokio runtime error: {0}")]
    Codec(#[from] bishop::codec::Error),
    #[error("received unexpected message from server")]
    UnexpectedMessage,
    #[error("server closed connection unexpectedly")]
    UnexpectedEOF,
}
