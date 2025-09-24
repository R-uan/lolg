use std::{net::Ipv4Addr, sync::Arc};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

struct LogClient {
    pub stream: Arc<RwLock<TcpStream>>,
}

pub struct Lolg {
    socket: TcpListener,
    pub running: Arc<RwLock<bool>>,
    clients: Arc<RwLock<Vec<LogClient>>>,
}

impl Lolg {
    pub async fn init(port: u16) -> Self {
        let host = Ipv4Addr::new(127, 0, 0, 1);
        let listener = TcpListener::bind((host, port)).await
    }
}
