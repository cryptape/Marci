use crate::models::{NetworkType, Peer};
use regex::Regex;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::string::String;
use std::time::{Duration, SystemTime};
use tokio_postgres::{Client, Row};

// Define a function to query the database for peers and their location information
pub(crate) async fn get_peers(
    network: NetworkType,
    offline_min: u64,
    unknown_offline_min: u64,
    client: &Client,
) -> Result<Vec<Peer>, tokio_postgres::Error> {
    let mut peers = Vec::new();

    for row in query_for_peers(&client, network, offline_min, false).await? {
        peers.push(process_row(&row, false));
    }

    for row in query_for_peers(&client, network, unknown_offline_min, true).await? {
        peers.push(process_row(&row, true));
    }
    Ok(peers)
}


async fn query_for_peers(client: &Client, network: NetworkType, offline_time: u64, empty_version: bool) -> Result<Vec<Row>, tokio_postgres::Error> {
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
WHERE peer.time >= NOW() - INTERVAL '{} min'
AND {}
ORDER BY peer.peer_id, peer.time, peer.id",
        main_scheme, main_scheme, offline_time, if empty_version { "peer.version = ''" } else { "peer.version <> ''"}
    );
    Ok(client.query(query.as_str(), &[]).await?)
}


fn process_row(row: &Row, is_unknown: bool) -> Peer {
    let version: Option<String> = row.get(2);

    let version_short = if is_unknown {
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

    Peer {
        id: row.get(0),
        version: version.unwrap_or(String::new()),
        version_short,
        last_seen: Some(row.get(3)),
        country: row.get(5),
        city: row.get(6),
        latitude,
        longitude,
        node_type: row.get(10),
    }
}
