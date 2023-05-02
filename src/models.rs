use serde::{Deserialize, Serialize};
use std::time::SystemTime;
// Define a struct to represent a peer with country and city information
#[derive(Serialize, Deserialize)]
pub struct Peer {
    pub(crate) id: i32,
    pub(crate) ip: String,
    pub(crate) version: String,
    pub(crate) last_seen: Option<SystemTime>,
    pub(crate) country: Option<String>,
    pub(crate) city: Option<String>,
}

pub(crate) enum NetworkType {
    Mirana,
    Pudge,
}

impl From<String> for NetworkType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "mirana" | "main" | "ckb" => NetworkType::Mirana,
            "pudge" | "test" | "ckb_test" => NetworkType::Pudge,
            _ => NetworkType::Mirana,
        }
    }
}

#[derive(Deserialize, Default)]
pub struct QueryParams {
    #[serde(default = "default_network")]
    pub(crate) network: String,
    #[serde(default = "default_timeout")]
    pub(crate) offline_timeout: u64,
}

fn default_network() -> String {
    "mirana".to_string()
}

fn default_timeout() -> u64 {
    30
}
