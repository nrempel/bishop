use std::time::Duration;

use clap::{command, Parser};
use thiserror::Error;
use tokio::process::Command;
use tokio::time;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let cmd = Command::new(args.command);

    let mut monitor = Monitor::new(cmd).max_restarts(args.max_restarts);
    match monitor.run().await {
        Ok(()) => println!("process exited successfully"),
        Err(e) => eprintln!("Monitoring error: {e}"),
    }
}

struct Monitor {
    command: Command,
    max_restarts: Option<usize>,
}

impl Monitor {
    fn new(command: Command) -> Self {
        Monitor {
            command,
            max_restarts: None,
        }
    }

    fn max_restarts(mut self, max_restarts: usize) -> Self {
        self.max_restarts = Some(max_restarts);
        self
    }

    async fn run(&mut self) -> Result<(), MonitorError> {
        let mut restart_count = 0;

        loop {
            let mut child = self.command.spawn()?;
            let ecode = child.wait().await?.code();

            match ecode {
                Some(0) => return Ok(()),
                _ => println!("restarting..."),
            }

            if let Some(max_restarts) = self.max_restarts {
                if restart_count >= max_restarts {
                    return Err(MonitorError::MaxRestartsExceeded);
                }
            }

            restart_count += 1;
            time::sleep(Duration::from_secs(1)).await;
        }
    }
}

#[derive(Debug, Error)]
enum MonitorError {
    #[error("failed to run command: {0}")]
    CommandError(#[from] std::io::Error),
    #[error("maximum restart attempts reached")]
    MaxRestartsExceeded,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    command: String,
    #[clap(long, default_value = "3")]
    max_restarts: usize,
}
