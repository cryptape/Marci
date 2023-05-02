use crate::models::{NetworkType, Peer};
use std::time::{Duration, SystemTime};
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
    let query = format!("SELECT peer.id, peer.ip, peer.version, peer.time as last_seen, ipinfo.country, ipinfo.city FROM {0}.peer LEFT JOIN {0}.ipinfo ON peer.ip = ipinfo.ip ORDER BY peer.id",main_scheme);
    let rows = client.query(query.as_str(), &[]).await?;
    let mut peers = Vec::new();
    for row in rows {
        let last_seen: SystemTime = row.get(3);
        if last_seen.elapsed().unwrap() > Duration::from_secs(offline_min * 60) {
            continue;
        }

        let peer = Peer {
            id: row.get(0),
            ip: row.get(1),
            version: row.get(2),
            last_seen: Some(row.get(3)),
            country: row.get(4),
            city: row.get(5),
        };
        peers.push(peer);
    }
    Ok(peers)
}
