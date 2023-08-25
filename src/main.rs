use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

use bishop::daemon::Daemon;
use clap::Parser;
use daemonize::Daemonize;
use thiserror::Error;
use tokio::runtime::Runtime;

fn main() -> Result<(), Error> {
    let args = Args::parse();
    match args.cmd {
        Commands::Up(args) => up(args),
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

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(Parser)]
enum Commands {
    Up(Up),
}

#[derive(Parser)]
struct Up {
    #[clap(long, default_value = "/tmp/bishop.pid")]
    pid_file: PathBuf,
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
}
