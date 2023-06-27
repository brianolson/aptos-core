// Copyright © Aptos Foundation

use std::sync::Arc;
use std::time::{Duration, Instant};
use aptos_metrics_core::{IntCounterVec, register_int_counter_vec};
use aptos_config::config::NodeConfig;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use aptos_network::application::interface::{NetworkClient, NetworkClientInterface, NetworkServiceEvents};
use aptos_time_service::{TimeService, TimeServiceTrait};
use aptos_network::protocols::network::Event;
use aptos_network::protocols::rpc::error::RpcError;
use futures::stream::{StreamExt,FuturesUnordered};
use futures::channel::oneshot::Sender;
use tokio::runtime::Handle;
use tokio::sync::RwLock;
use tokio::select;
use aptos_config::network_id::{NetworkId, PeerNetworkId};
use aptos_logger::{info,warn};
use aptos_network::protocols::wire::handshake::v1::ProtocolId;
// use aptos_network::protocols::wire::handshake::v1::ProtocolId::BenchmarkDirectSend;
use aptos_types::account_address::AccountAddress;
use aptos_types::PeerId;
use bytes::Bytes;
use rand::Rng;
use rand::rngs::OsRng;
use std::ops::DerefMut;
// use futures::future::MaybeDone::Future;


#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum BenchmarkMessage {
    DataSend(BenchmarkDataSend),
    DataReply(BenchmarkDataReply),
}

#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct BenchmarkDataSend {
    pub request_counter: u64, // A monotonically increasing counter to verify responses
    pub send_micros: i64, // micro seconds since some epoch at a moment just before this message is sent
    pub data: Vec<u8>, // A vector of bytes to send in the request; zero length in reply
}

#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct BenchmarkDataReply {
    pub request_counter: u64, // A monotonically increasing counter to verify responses
    pub send_micros: i64, // micro seconds since some epoch at a moment just before this message is sent
    pub your_send_micros: i64, // the send_micros from the previous message
}

/// Counter for pending network events to the monitoring service (server-side)
pub static PENDING_BENCHMARK_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "aptos_benchmark_pending_network_events",
        "Counters for pending network events for benchmarking",
        &["state"]
    ).unwrap()
});
//
// pub struct BenchmarkService {
//     time_service: TimeService,
//     shared : Arc<RwLock<BenchmarkSharedState>>,
// }
//
// impl BenchmarkService {
//     // pub async fn run(
//     //     network_client : NetworkClient<BenchmarkMessage>,
//     //     events: NetworkServiceEvents<BenchmarkMessage>,
//     //     time_service: TimeService,
//     //     shared : Arc<RwLock<BenchmarkSharedState>>,
//     // ) {
//     //     let bs = BenchmarkService{
//     //         time_service,
//     //         shared,
//     //     };
//     //     bs.service_loop(network_client, events).await;
//     // }
//
//     // all-in-one, get events from NetworkServiceEvents and handle them.
//     async fn service_loop(
//         mut self,
//         network_client : NetworkClient<BenchmarkMessage>,
//         network_requests: NetworkServiceEvents<BenchmarkMessage>,
//     ) {
//         let network_events: Vec<_> = network_requests
//             .into_network_and_events()
//             .into_iter()
//             .map(|(network_id, events)| events.map(move |event| (network_id, event)))
//             .collect();
//         let mut network_events = futures::stream::select_all(network_events).fuse();
//
//         loop {
//             let (network_id, event) = match network_events.next().await {
//                 None => { return; }  // fused stream will never return more
//                 Some(x) => { x }
//             };
//             match event {
//                 Event::Message(peer_id, wat) => {
//                     let msg_wrapper: BenchmarkMessage = wat;
//                     // TODO: counters, note blob size and increment message counter
//                     self.handle_direct(&network_client, network_id, peer_id, msg_wrapper).await;
//                 }
//                 Event::RpcRequest(peer_id, msg_wrapper, protocol_id, sender) => {
//                     self.handle_rpc(peer_id, msg_wrapper, protocol_id, sender).await;
//                 }
//                 Event::NewPeer(_) => {}  // don't care
//                 Event::LostPeer(_) => {}  // don't care
//             }
//         }
//     }
//
//     async fn handle_direct(
//         &mut self,
//         network_client : &NetworkClient<BenchmarkMessage>,
//         network_id: NetworkId,
//         peer_id: AccountAddress,
//         msg_wrapper: BenchmarkMessage
//     ) {
//         handle_direct(network_client, network_id, peer_id, msg_wrapper, self.time_service.clone(), self.shared.clone()).await;
//         // match msg_wrapper {
//         //     BenchmarkMessage::DataSend(send) => {
//         //         let reply = BenchmarkDataReply {
//         //             request_counter: send.request_counter,
//         //             send_micros: self.time_service.now_unix_time().as_micros() as i64,
//         //             your_send_micros: send.send_micros,
//         //         };
//         //         let result = network_client.send_to_peer(BenchmarkMessage::DataReply(reply), PeerNetworkId::new(network_id, peer_id));
//         //         if let Err(_) = result {
//         //             // TODO: log, counter
//         //         }
//         //     }
//         //     BenchmarkMessage::DataReply(reply) => {
//         //         let receive_time = self.time_service.now_unix_time().as_micros() as i64;
//         //         let rec = {
//         //             let reader = self.shared.read().await;
//         //             reader.find(reply.request_counter)
//         //         };
//         //         if rec.request_counter == reply.request_counter {
//         //             info!("pmd[{}] {} bytes at {} in {} micros", rec.request_counter, rec.bytes_sent, receive_time, receive_time - rec.send_micros);
//         //         } else {
//         //             info!("pmd[{}] unk bytes in > {} micros", reply.request_counter, receive_time - rec.send_micros)
//         //         }
//         //     }
//         // }
//     }
//
//     async fn handle_rpc(&mut self, _peer_id: AccountAddress, msg_wrapper: BenchmarkMessage, protocol_id: ProtocolId, sender: Sender<Result<Bytes, RpcError>>) {
//         handle_rpc(_peer_id, msg_wrapper, protocol_id, self.time_service.clone(), sender).await;
//         // match msg_wrapper {
//         //     BenchmarkMessage::DataSend(send) => {
//         //         let reply = BenchmarkDataReply {
//         //             request_counter: send.request_counter,
//         //             send_micros: self.time_service.now_unix_time().as_micros() as i64,
//         //             your_send_micros: send.send_micros,
//         //         };
//         //         let reply = BenchmarkMessage::DataReply(reply);
//         //         let reply_bytes = match protocol_id.to_bytes(&reply) {
//         //             Ok(rb) => { rb }
//         //             Err(_) => {return} // TODO: counter, log
//         //         };
//         //         let reply_bytes: Bytes = reply_bytes.into();
//         //         let result = sender.send(Ok(reply_bytes));
//         //         if let Err(_) = result {
//         //             // TODO: log, counter
//         //         }
//         //     }
//         //     BenchmarkMessage::DataReply(_) => {
//         //         // TODO: ERROR, log, counter, this should come back at the point of the RPC call, not here
//         //     }
//         // }
//     }
// }

// Get messages from the network and quickly shuffle them to N threads of workers.
async fn source_loop(
    network_requests: NetworkServiceEvents<BenchmarkMessage>,
    work : async_channel::Sender<(NetworkId,Event<BenchmarkMessage>)>,
) {
    let network_events: Vec<_> = network_requests
        .into_network_and_events()
        .into_iter()
        .map(|(network_id, events)| events.map(move |event| (network_id, event)))
        .collect();
    let mut network_events = futures::stream::select_all(network_events).fuse();

    loop {
        match network_events.next().await {
            None => {
                // fused stream will never return more
                work.close();
                return;
            }
            Some(x) => {
                match work.send(x).await {
                    Ok(_) => {},
                    Err(se) => {
                        warn!("benchmark source_loop work send: {}", se);
                    }
                } // TODO: log, counter
            }
        };
    }
}

async fn handle_direct(
    network_client : &NetworkClient<BenchmarkMessage>,
    network_id: NetworkId,
    peer_id: AccountAddress,
    msg_wrapper: BenchmarkMessage,
    time_service: TimeService,
    shared : Arc<RwLock<BenchmarkSharedState>>,
) {
    match msg_wrapper {
        BenchmarkMessage::DataSend(send) => {
            let reply = BenchmarkDataReply {
                request_counter: send.request_counter,
                send_micros: time_service.now_unix_time().as_micros() as i64,
                your_send_micros: send.send_micros,
            };
            let result = network_client.send_to_peer(BenchmarkMessage::DataReply(reply), PeerNetworkId::new(network_id, peer_id));
            if let Err(_) = result {
                // TODO: log, counter
            }
        }
        BenchmarkMessage::DataReply(reply) => {
            let receive_time = time_service.now_unix_time().as_micros() as i64;
            let rec = {
                let reader = shared.read().await;
                reader.find(reply.request_counter)
            };
            if rec.request_counter == reply.request_counter {
                info!("benchmark ds [{}] {} bytes at {} in {} micros", rec.request_counter, rec.bytes_sent, receive_time, receive_time - rec.send_micros);
            } else {
                info!("benchmark ds [{}] unk bytes in > {} micros", reply.request_counter, receive_time - rec.send_micros)
            }
        }
    }
}

async fn handle_rpc(_peer_id: AccountAddress, msg_wrapper: BenchmarkMessage, protocol_id: ProtocolId, time_service: TimeService, sender: Sender<Result<Bytes, RpcError>>) {
    match msg_wrapper {
        BenchmarkMessage::DataSend(send) => {
            let reply = BenchmarkDataReply {
                request_counter: send.request_counter,
                send_micros: time_service.now_unix_time().as_micros() as i64,
                your_send_micros: send.send_micros,
            };
            let reply = BenchmarkMessage::DataReply(reply);
            let reply_bytes = match protocol_id.to_bytes(&reply) {
                Ok(rb) => { rb }
                Err(_) => {return} // TODO: counter, log
            };
            let reply_bytes: Bytes = reply_bytes.into();
            let result = sender.send(Ok(reply_bytes));
            if let Err(_) = result {
                // TODO: log, counter
            }
        }
        BenchmarkMessage::DataReply(_) => {
            // TODO: ERROR, log, counter, this should come back at the point of the RPC call, not here
        }
    }
}

/// handle work split out by source_loop()
async fn handler_thread(
    node_config: NodeConfig,
    network_client : NetworkClient<BenchmarkMessage>,
    work_rx : async_channel::Receiver<(NetworkId, Event<BenchmarkMessage>)>,
    time_service: TimeService,
    shared : Arc<RwLock<BenchmarkSharedState>>,
) {
    let config = node_config.benchmark.unwrap();
    loop {
        let (network_id, event) = match work_rx.recv().await {
            Ok(v) => v,
            Err(_) => {
                // RecvError means source was closed, we're done here.
                return;
            },
        };
        match event {
            Event::Message(peer_id, wat) => {
                let msg_wrapper: BenchmarkMessage = wat;
                // TODO: counters, note blob size and increment message counter
                //self.handle_direct(&network_client, network_id, peer_id, msg_wrapper).await;
                handle_direct(&network_client, network_id, peer_id, msg_wrapper, time_service.clone(), shared.clone()).await;
            }
            Event::RpcRequest(peer_id, msg_wrapper, protocol_id, sender) => {
                //self.handle_rpc(peer_id, msg_wrapper, protocol_id, sender).await;
                handle_rpc(peer_id, msg_wrapper, protocol_id, time_service.clone(), sender).await;
            }
            Event::NewPeer(wat) => {
                if config.enable_direct_send_testing {
                    Handle::current().spawn(direct_sender(
                        node_config.clone(),
                        network_client.clone(),
                        time_service.clone(),
                        network_id,
                        wat.remote_peer_id,
                        shared.clone(),
                    ));
                }
                if config.enable_rpc_testing {
                    Handle::current().spawn(rpc_sender(
                        node_config.clone(),
                        network_client.clone(),
                        time_service.clone(),
                        network_id,
                        wat.remote_peer_id,
                        shared.clone(),
                    ));
                }
            }
            Event::LostPeer(_) => {}  // don't care
        }
    }
}

/// run_benchmark_service() does not return, it should be called by .spawn()
pub async fn run_benchmark_service(
    node_config: NodeConfig,
    // benchmark_service_threads: Option<usize>,
    network_client : NetworkClient<BenchmarkMessage>,
    network_requests: NetworkServiceEvents<BenchmarkMessage>,
    time_service: TimeService,
    shared : Arc<RwLock<BenchmarkSharedState>>,
) {
    let config = node_config.benchmark.unwrap();
    let benchmark_service_threads = config.benchmark_service_threads;
    let num_threads = match benchmark_service_threads {
        Some(x) => x,
        None => {
            match std::thread::available_parallelism() {
                Ok(val) => val.get(),
                Err(_) => 1,
            }
        },
    };
    let (work_sender, work_receiver) = async_channel::bounded(num_threads * 2);
    let runtime_handle = Handle::current();
    let source_thread = runtime_handle.spawn(source_loop(network_requests, work_sender));
    let mut handlers = vec![];
    for _ in 0..num_threads {
        handlers.push(runtime_handle.spawn(handler_thread(
            node_config.clone(),
            network_client.clone(),
            work_receiver.clone(),
            time_service.clone(),
            shared.clone())));
    }
    if let Err(err) = source_thread.await {
        warn!("benchmark source_thread join: {}", err);
    }
    for hai in handlers {
        if let Err(err) = hai.await {
            warn!("benchmark handler_thread join: {}", err);
        }
    }
}

pub async fn direct_sender(
    node_config: NodeConfig,
    network_client : NetworkClient<BenchmarkMessage>,
    time_service: TimeService,
    network_id: NetworkId,
    peer_id: PeerId,
    shared : Arc<RwLock<BenchmarkSharedState>>,
) {
    let config = node_config.benchmark.unwrap();
    let interval = Duration::from_nanos(1_000_000_000/config.direct_send_per_second);
    let ticker = time_service.interval(interval);
    futures::pin_mut!(ticker);
    // random payload filler
    let data_size = config.direct_send_data_size;
    let mut blob = Vec::<u8>::with_capacity(data_size);
    let mut rng = OsRng;
    for _ in 0..data_size {
        blob.push(rng.gen());
    }

    let mut counter : u64 = rng.gen();

    let mut next_blab : i64 = 0;

    loop {
        ticker.next().await;

        counter += 1;
        {
            // tweak the random payload a little on every send
            let counter_bytes: [u8; 8] = counter.to_le_bytes();
            let (dest, _) = blob.deref_mut().split_at_mut(8);
            dest.copy_from_slice(&counter_bytes);
        }

        let nowu = time_service.now_unix_time().as_micros() as i64;
        let msg = BenchmarkDataSend {
            request_counter: counter,
            send_micros: nowu,
            data: blob.clone(),
        };
        {
            shared.write().await.set(SendRecord{
                request_counter: counter,
                send_micros: nowu,
                bytes_sent: blob.len(),
            })
        }
        let wrapper = BenchmarkMessage::DataSend(msg);
        let start_send : Instant = time_service.now();
        let result = network_client.send_to_peer(wrapper, PeerNetworkId::new(network_id, peer_id));
        let send_end : Instant = time_service.now();
        if let Err(err) = result {
            info!("benchmark [{},{}] direct send err: {}", network_id, peer_id, err);
            // TODO: some error limit, or error-per-second limit, or specifically detect unrecoverable errors
            return;
        } else {
            let send_dt = send_end.duration_since(start_send);
            info!("benchmark direct[{}] at {} µs, took {} µs", counter, nowu, send_dt.as_micros());
        }

        if nowu > next_blab {
            info!("benchmark ds counter={}", counter);
            next_blab = nowu + 20_000;
        }
    }
}
//
// pub async fn inner_rpc_send<T>(
//     network_client : &NetworkClient<BenchmarkMessage>,
//     counter: u64,
//     blob: &mut Vec<u8>,
//     time_service: &TimeService,
//     network_id: NetworkId,
//     peer_id: PeerId,
//     shared : Arc<RwLock<BenchmarkSharedState>>,
//     open_rpcs: &mut FuturesUnordered<T>
// ) -> i64 {
//     {
//         // tweak the random payload a little on every send
//         let counter_bytes: [u8; 8] = counter.to_le_bytes();
//         let (dest, _) = blob.deref_mut().split_at_mut(8);
//         dest.copy_from_slice(&counter_bytes);
//     }
//
//     let nowu = time_service.now_unix_time().as_micros() as i64;
//     let msg = BenchmarkDataSend {
//         request_counter: counter,
//         send_micros: nowu,
//         data: blob.clone(),
//     };
//     {
//         shared.write().await.set(SendRecord{
//             request_counter: counter,
//             send_micros: nowu,
//             bytes_sent: blob.len(),
//         })
//     }
//     let wrapper = BenchmarkMessage::DataSend(msg);
//     //let start_send : Instant = time_service.now();
//     // let result = network_client.send_to_peer(wrapper, PeerNetworkId::new(network_id, peer_id));
//     let result = network_client.send_to_peer_rpc(wrapper, Duration::from_secs(10), PeerNetworkId::new(network_id, peer_id));
//     //let send_end : Instant = time_service.now(); We probably don't care how long it takes to _start_ an rpc
//     open_rpcs.push(result);
//     return nowu;
// }

pub async fn rpc_sender(
    node_config: NodeConfig,
    network_client : NetworkClient<BenchmarkMessage>,
    time_service: TimeService,
    network_id: NetworkId,
    peer_id: PeerId,
    shared : Arc<RwLock<BenchmarkSharedState>>,
) {
    let config = node_config.benchmark.unwrap();
    let interval = Duration::from_nanos(1_000_000_000/config.rpc_per_second);
    let ticker = time_service.interval(interval);
    futures::pin_mut!(ticker);
    // random payload filler
    let data_size = config.rpc_data_size;
    let mut blob = Vec::<u8>::with_capacity(data_size);
    let mut rng = OsRng;
    for _ in 0..data_size {
        blob.push(rng.gen());
    }

    let mut counter : u64 = rng.gen();

    let mut next_blab : i64 = 0;

    //let mut open_rpcs = vec![];
    let mut open_rpcs = FuturesUnordered::new();

    // let mut tick_missed = false;

    loop {
        select! {
            _ = ticker.next() => {
                if open_rpcs.len() >= config.rpc_in_flight {
                    // tick_missed = true;
                    continue;
                }
                // do rpc send
                counter += 1;
                //let nowu = inner_rpc_send(&network_client, counter, &mut blob, &time_service, network_id, peer_id, shared.clone(), &mut open_rpcs).await;
                {
                    // tweak the random payload a little on every send
                    let counter_bytes: [u8; 8] = counter.to_le_bytes();
                    let (dest, _) = blob.deref_mut().split_at_mut(8);
                    dest.copy_from_slice(&counter_bytes);
                }

                let nowu = time_service.now_unix_time().as_micros() as i64;
                let msg = BenchmarkDataSend {
                    request_counter: counter,
                    send_micros: nowu,
                    data: blob.clone(),
                };
                {
                    shared.write().await.set(SendRecord{
                        request_counter: counter,
                        send_micros: nowu,
                        bytes_sent: blob.len(),
                    })
                }
                let wrapper = BenchmarkMessage::DataSend(msg);
                //let start_send : Instant = time_service.now();
                // let result = network_client.send_to_peer(wrapper, PeerNetworkId::new(network_id, peer_id));
                let result = network_client.send_to_peer_rpc(wrapper, Duration::from_secs(10), PeerNetworkId::new(network_id, peer_id));
                //let send_end : Instant = time_service.now(); We probably don't care how long it takes to _start_ an rpc
                open_rpcs.push(result);

                if nowu > next_blab {
                    info!("benchmark ds counter={}", counter);
                    next_blab = nowu + 20_000;
                }
            }
            result = open_rpcs.next() => {
                let result = match result {
                    Some(subr) => {subr}
                    None => {
                        continue
                    }
                };
                // handle rpc result
                let nowu = time_service.now_unix_time().as_micros() as i64;
                match result {
                    Err(err) => {
            info!("benchmark [{},{}] rpc send err: {}", network_id, peer_id, err);
            // TODO: some error limit, or error-per-second limit, or specifically detect unrecoverable errors
            return;
                    }
                    Ok(msg_wrapper) => {
                        if let BenchmarkMessage::DataReply(msg) = msg_wrapper {
                            let send_dt = nowu - msg.your_send_micros;
                            //let send_dt = send_end.duration_since(start_send);
                            info!("benchmark [{}] rpc at {} µs, took {} µs", counter, nowu, send_dt);
                        } else {
                            info!("benchmark [{}] rpc wat", counter);
                        }
                    }
                }
                // TODO: if tick_missed {start a new send right now}
            }
        }
    };
}

pub struct BenchmarkSharedState {
    // Circular buffer of sent records
    sent: Vec<SendRecord>,
    // sent[sent_pos] is the next index to write
    sent_pos: usize,
}

impl BenchmarkSharedState {
    pub fn new() -> Self {
        BenchmarkSharedState {
            sent: Vec::with_capacity(10000), // TODO: constant or config
            sent_pos: 0,
        }
    }

    pub fn set(&mut self, sent: SendRecord) {
        if self.sent.len() < self.sent.capacity() {
            self.sent.push(sent);
        } else {
            self.sent[self.sent_pos] = sent;
        }
        self.sent_pos = (self.sent_pos + 1) % self.sent.capacity();
    }

    /// return the record for the request_counter, or {0, oldest send_micros}
    /// Option<SendRecord> might seem like it would make sense, but we use the send_micros field to return the oldest known message time when we don't find a request_counter match.
    pub fn find(&self, request_counter: u64) -> SendRecord {
        if self.sent.len() == 0 {
            return SendRecord{request_counter: 0, send_micros: 0, bytes_sent: 0};
        }
        let mut oldest = self.sent[0].send_micros;
        let capacity = self.sent.len();
        for i in 0..capacity {
            let pos = (self.sent_pos + capacity - (1+i)) % capacity;
            let rec = self.sent[pos].clone();
            if rec.request_counter == request_counter {
                return rec;
            }
            if rec.send_micros < oldest {
                oldest = rec.send_micros;
            }
        }
        SendRecord{request_counter: 0, send_micros: oldest, bytes_sent: 0}
    }
}

#[derive(Clone)]
pub struct SendRecord {
    pub request_counter: u64,
    pub send_micros: i64,
    pub bytes_sent: usize,
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
