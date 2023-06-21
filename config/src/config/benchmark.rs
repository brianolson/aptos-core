// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use serde::{Serialize,Deserialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BenchmarkConfig {
    pub enable_benchmark_service: bool,
    pub max_network_channel_size: u64, // Max num of pending network messages
    pub benchmark_service_threads: Option<usize>, // Number of kernel threads for tokio runtime. None default for num-cores.
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enable_benchmark_service: false,
            max_network_channel_size: 1000,
            benchmark_service_threads: None,
        }
    }
}
