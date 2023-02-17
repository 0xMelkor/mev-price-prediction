use std::env;

use ethers::types::Address;

pub struct Config {
    pub http_url: String,
    pub ws_url: String,
    pub aave_protocol_data_provider: Address,
    pub aave_oracle: Address,
}

impl Config {
    pub fn new() -> Self {
        Self {
            http_url: Self::get_env("RPC_HTTP_URL"),
            ws_url: Self::get_env("RPC_WS_URL"),
            aave_protocol_data_provider: Self::addr("0x057835Ad21a177dbdd3090bB1CAE03EaCF78Fc6d"),
            aave_oracle: Self::addr("0xA50ba011c48153De246E5192C8f9258A2ba79Ca9"),
        }
    }

    fn get_env(key: &str) -> String {
        let msg = format!("Cannot find env {key}");
        env::var(key).expect(&msg)
    }

    fn addr(hex: &str) -> Address {
        hex.parse().expect("Invalid address")
    }
}
