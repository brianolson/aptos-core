// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use serde::{Serialize,Deserialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BenchmarkConfig {
    pub enable_benchmark_service: bool,
    pub max_network_channel_size: u64, // Max num of pending network messages
    pub benchmark_service_threads: Option<usize>, // Number of kernel threads for tokio runtime. None default for num-cores.

    pub enable_direct_send_testing: bool, // Whether or not to enable direct send test mode
    pub direct_send_data_size: u64,       // The amount of data to send in each request
    pub direct_send_per_second: u64,      // The interval (microseconds) between requests
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enable_benchmark_service: false,
            max_network_channel_size: 1000,
            benchmark_service_threads: None,

            enable_direct_send_testing: true,
            direct_send_data_size: 100 * 1024,    // 100 KB
            direct_send_per_second: 1_000,
        }
    }
}
