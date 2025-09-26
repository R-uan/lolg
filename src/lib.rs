mod error;
mod log_entry;

use std::sync::Arc;

use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf, sync::RwLock};

use crate::{
    error::Error,
    log_entry::{Level, LogEntry},
};

struct LogClient {
    pub write: Arc<RwLock<OwnedWriteHalf>>,
}

impl LogClient {
    pub fn new(write_half: OwnedWriteHalf) -> Self {
        let write = Arc::new(RwLock::new(write_half));
        return LogClient { write };
    }

    pub async fn send(&self, bytes: &Vec<u8>) -> bool {
        let mut write_guard = self.write.write().await;
        write_guard.write(bytes).await.is_ok()
    }
}

pub struct Lolg {
    debugging: bool,
    pub running: Arc<RwLock<bool>>,
    socket: tokio::net::TcpListener,
    clients: Arc<RwLock<Vec<LogClient>>>,
}

impl Lolg {
    /// Initializes a Lolg instance with a TcpListener socket hosted locally.
    /// - `port`: port that the socket will be open at.
    pub async fn init(port: u16, debugging: bool) -> Result<Arc<Self>, Error> {
        let host = std::net::Ipv4Addr::new(127, 0, 0, 1);
        let listener = tokio::net::TcpListener::bind((host, port))
            .await
            .map_err(|_| Error::SocketFailed(format!("{host}/{port}")))?;

        let lolg = Self {
            debugging,
            socket: listener,
            running: Arc::new(RwLock::new(false)),
            clients: Arc::new(RwLock::new(Vec::new())),
        };

        lolg.debug(&format!("Lolg running on port: {port}")).await;

        return Ok(Arc::new(lolg));
    }

    pub async fn listen(self: Arc<Self>) {
        let lolg = Arc::clone(&self);
        *lolg.running.write().await = true;
        tokio::spawn(async move {
            while *lolg.running.read().await {
                match lolg.socket.accept().await {
                    Err(_) => {}
                    Ok((stream, addr)) => {
                        let (_, write) = stream.into_split();
                        let client = LogClient::new(write);
                        lolg.clients.write().await.push(client);

                        if self.debugging {
                            let msg = format!("New log client has been connected ({addr})");
                            let entry = LogEntry::new(Level::Debug, &msg);
                            println!("{entry}");
                        }
                    }
                }
            }
        });
    }

    pub async fn debug(&self, msg: &str) {
        self.send(Level::Debug, msg).await;
    }

    pub async fn info(&self, msg: &str) {
        self.send(Level::Info, msg).await;
    }

    pub async fn warn(&self, msg: &str) {
        self.send(Level::Warning, msg).await;
    }

    pub async fn error(&self, msg: &str) {
        self.send(Level::Error, msg).await;
    }

    /// - Creates a log entry and sends it through the TCP stream as bytes.
    /// - Calls the `cleanup` function to remove clients that the packet couldn't be sent to.
    /// - If the debug option is on, logs the amount of successful deliveries.
    async fn send(&self, level: Level, msg: &str) {
        let entry = LogEntry::new(level, msg);
        if self.debugging {
            println!("{entry}");
        }
        let mut cleanup: Vec<usize> = Vec::new();
        let encode = entry.bytes();
        let clients = self.clients.read().await;
        if clients.len() > 0 {
            for index in 0..clients.len() {
                let client = &clients[index];
                if !client.send(&encode).await {
                    cleanup.push(index);
                }
            }

            if self.debugging {
                let msg = format!(
                    "Entry has been sent to {}/{} clients",
                    clients.len() - cleanup.len(),
                    &clients.len()
                );
                let debug = LogEntry::new(Level::Debug, &msg);
                println!("{debug}");
            }

            self.cleanup(&cleanup).await;
        }
    }

    /// - Removes target client(s) from the client pool.
    /// - `targets` vec of indexes to be removed from `clients`
    pub async fn cleanup(&self, targets: &Vec<usize>) {
        let mut clients = self.clients.write().await;
        for index in targets {
            clients.remove(*index);
        }

        if self.debugging {
            let msg = format!("{} clients have cleaned up", targets.len());
            let debug = LogEntry::new(Level::Debug, &msg);
            println!("{debug}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init() {
        let lolg = Lolg::init(3001, true).await;
        assert!(lolg.is_ok(), "Expected an OK, but got an Err")
    }
}
