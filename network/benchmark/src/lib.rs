// Copyright © Aptos Foundation

use aptos_metrics_core::{IntCounterVec,register_int_counter_vec};
use aptos_config::config::NodeConfig;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use aptos_network::application::interface::{NetworkClient, NetworkClientInterface, NetworkServiceEvents};
use aptos_time_service::{TimeService, TimeServiceTrait};


#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum BenchmarkMessage {
    DataSend(BenchmarkDataSend)
}

#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct BenchmarkDataSend {
    pub request_counter: u64, // A monotonically increasing counter to verify responses
    pub send_micros: i64, // micro seconds since some epoch at a moment just before this message is sent
    pub your_send_micros: i64, // If this message is a reply, the send_micros from the previous message
    pub is_reply: bool, // false for an iniating message, true for a reply
    pub data: Vec<u8>, // A vector of bytes to send in the request; zero length in reply
}

/// Counter for pending network events to the monitoring service (server-side)
pub static PENDING_BENCHMARK_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "aptos_benchmark_pending_network_events",
        "Counters for pending network events for benchmarking",
        &["state"]
    )
        .unwrap()
});

pub struct BenchmarkService {

}

impl BenchmarkService {
    pub async fn run(
        network_client : NetworkClient<BenchmarkMessage>,
        events: NetworkServiceEvents<BenchmarkMessage>,
        time_service: TimeService,
    ) {

    }
}

pub async fn run_benchmark_service(
    network_client : NetworkClient<BenchmarkMessage>,
    events: NetworkServiceEvents<BenchmarkMessage>,
    time_service: TimeService,
) {
    //let mut bs : BenchmarkService;
    //bs.run(network_client, events, time_service)
    for (network_id, network_events) in events.into_network_and_events() {

    }
    loop {
    }
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
