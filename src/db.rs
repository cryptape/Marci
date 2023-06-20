use crate::models::{NetworkType, Peer};
use regex::Regex;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::string::String;
use std::time::{Duration, SystemTime};
use tokio_postgres::Client;

// Define a function to query the database for peers and their location information
pub(crate) async fn get_peers(
    network: NetworkType,
    offline_min: u64,
    unknown_offline_min: u64,
    client: &Client,
) -> Result<Vec<Peer>, tokio_postgres::Error> {
    let main_scheme = match network {
        NetworkType::Mirana => "ckb",
        NetworkType::Pudge => "ckb_testnet",
    };

    let query = format!(
        "
SELECT DISTINCT ON (peerID)
    peer.id,
    peer.ip,
    peer.version,
    peer.time as last_seen,
    peer.address,
    ipinfo.country,
    ipinfo.city,
    ipinfo.latitude as latitude,
    ipinfo.longitude as longitude,
    peer.peer_id as peerID,
    peer.node_type
FROM {}.peer
JOIN {}.ipinfo AS ipinfo ON peer.ip = ipinfo.ip
ORDER BY peer.peer_id, (peer.address LIKE '/ip4/%') DESC, peer.time, peer.id",
        main_scheme, main_scheme
    );

    let rows = client.query(query.as_str(), &[]).await?;
    let mut peers = Vec::new();

    for row in rows {
        let last_seen: SystemTime = row.get(3);
        let version: Option<String> = row.get(2);

        if version.clone().unwrap_or_default().is_empty() {
            // unknown peer, use another timeout
            if last_seen.elapsed().unwrap() > Duration::from_secs(unknown_offline_min * 60) {
                continue;
            }
        } else {
            if last_seen.elapsed().unwrap() > Duration::from_secs(offline_min * 60) {
                continue;
            }
        }

        let version_short: String = if version.is_none() || version.clone().unwrap().is_empty() {
            "Unknown".to_string()
        } else {
            Regex::new(r"^(.*?)[^0-9.].*$")
                .unwrap()
                .captures(&version.clone().unwrap())
                .unwrap()[1]
                .to_owned()
        };

        let latitude: Option<Decimal> = row.get(7);
        let longitude: Option<Decimal> = row.get(8);

        let latitude: Option<f64> = if latitude.is_none() {
            None
        } else {
            latitude.unwrap().to_f64()
        };
        let longitude: Option<f64> = if longitude.is_none() {
            None
        } else {
            longitude.unwrap().to_f64()
        };

        let peer = Peer {
            id: row.get(0),
            //ip: row.get(1),
            version: version.unwrap_or(String::new()),
            version_short,
            last_seen: Some(row.get(3)),
            //address: row.get(4),
            country: row.get(5),
            city: row.get(6),
            latitude,
            longitude,
            node_type: row.get(10),
        };
        peers.push(peer);
    }
    Ok(peers)
}
