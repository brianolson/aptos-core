// Copyright © Aptos Foundation

use std::net::SocketAddr;
use tokio::net::TcpListener;

fn main() {
    println!("Hello, world!");
}

pub struct Network {
    peers: Vec<Peer>,

    listeners: TcpListener,
}

struct Listener {
    socket: TcpListener,
    policy: ConnectionType,
}

impl Listener {
    pub async fn new<A: tokio::net::ToSocketAddrs>(addr: A, policy: ConnectionType) -> std::io::Result<Self> {
        let mut socket = TcpListener::bind(addr).await?;
        Ok(Listener{
            socket,
            policy,
        })
    }

    pub async fn listen_loop(&mut self) {
        loop {
            match self.socket.accept().await {
                Ok((socket, addr)) => {
                    self.start_inbound_peer(socket,addr);
                },
                Err(e) => {}, // TODO: log
            }
        }
    }

    pub async fn start_inbound_peer(&mut self, stream: tokio::net::TcpStream, addr: SocketAddr) {

    }
}

pub struct Peer {
    socket: tokio::net::TcpStream,
    policy: ConnectionType,
}

pub enum ConnectionType {
    Unknown,
    Validator,
    VFN,
    PFN,
    Other,
}
