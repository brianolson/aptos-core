// Copyright © Aptos Foundation

use std::net::SocketAddr;
use std::str::FromStr;
use std::io::Result;
use tokio::net::TcpListener;
use aptos_types::network_address::NetworkAddress;
use aptos_network2::util::listen;

fn main() {
    match result_main() {
        Err(e) => {
            println!("err: {}", e);
        }
        Ok(_) => {
            println!("Ok!");
        }
    }
}

fn result_main() -> Result<()> {
    let na = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/8301").map_err(|e| Error::)?;
    let wat = listen(na)?;
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
