use crate::models::{NetworkType, Peer};
use std::time::{Duration, SystemTime};
use regex::Regex;
use tokio_postgres::Client;

// Define a function to query the database for peers and their location information
pub(crate) async fn get_peers(
    network: NetworkType,
    offline_min: u64,
    client: &Client,
) -> Result<Vec<Peer>, tokio_postgres::Error> {
    let main_scheme = match network {
        NetworkType::Mirana => "ckb",
        NetworkType::Pudge => "ckb_testnet",
    };
    let query = format!("SELECT DISTINCT ON (peer.address) peer.id, peer.ip, peer.version, peer.time as last_seen, peer.address, ipinfo.country, ipinfo.city FROM {}.peer LEFT JOIN {}.ipinfo ON peer.ip = ipinfo.ip ORDER BY peer.address, peer.id", main_scheme, main_scheme);
    let rows = client.query(query.as_str(), &[]).await?;
    let mut peers = Vec::new();
    for row in rows {
        let last_seen: SystemTime = row.get(3);
        if last_seen.elapsed().unwrap() > Duration::from_secs(offline_min * 60) {
            continue;
        }

        let version: String = row.get(2);
        let version_short = Regex::new(r"^(.*?)[^0-9.].*$").unwrap().captures(&version).unwrap()[1].to_owned();


        let peer = Peer {
            id: row.get(0),
            ip: row.get(1),
            version,
            version_short,
            last_seen: Some(row.get(3)),
            address: row.get(4),
            country: row.get(5),
            city: row.get(6),
        };
        peers.push(peer);
    }
    Ok(peers)
}
