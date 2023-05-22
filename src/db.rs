use crate::models::{NetworkType, Peer};
use std::time::{Duration, SystemTime};
use regex::Regex;
use tokio_postgres::{Client, types::FromSql};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

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

    let query = format!("
    SELECT
        peer.id,
        peer.ip,
        peer.version,
        peer.time as last_seen,
        peer.address
    FROM {}.peer
    ORDER BY peer.address, peer.id", main_scheme);

    let rows = client.query(query.as_str(), &[]).await?;
    let mut peers = Vec::new();

    for row in rows {
        let last_seen: SystemTime = row.get(3);
        if last_seen.elapsed().unwrap() > Duration::from_secs(offline_min * 60) {
            continue;
        }

        let version: String = row.get(2);
        let version_short = Regex::new(r"^(.*?)[^0-9.].*$").unwrap().captures(&version).unwrap()[1].to_owned();

        // Now, make a second query to fetch IP information for this peer
        let ip = row.get::<usize, String>(1);
        let ip_info_query = format!("
        SELECT
            ipinfo.country_code,
            ipinfo.city,
            ipinfo.latitude,
            ipinfo.longitude
        FROM common_info.ip_info AS ipinfo
        WHERE '{}' BETWEEN ipinfo.ip_range_start AND ipinfo.ip_range_end
        LIMIT 1", ip);
        let ip_info_rows = client.query(ip_info_query.as_str(), &[]).await?;

        if ip_info_rows.is_empty() {
            continue;
        }

        let ip_info_row = ip_info_rows.first().unwrap();

        let latitude : Option<Decimal> = ip_info_row.get(2);
        let longitude: Option<Decimal> = ip_info_row.get(3);

        let latitude: Option<f64> = latitude.unwrap_or_default().to_f64();
        let longitude: Option<f64> = longitude.unwrap_or_default().to_f64();

        let peer = Peer {
            id: row.get(0),
            ip: row.get(1),
            version,
            version_short,
            last_seen: Some(row.get(3)),
            address: row.get(4),
            country: ip_info_row.get(0),
            city: ip_info_row.get(1),
            latitude: latitude,
            longitude: longitude,
        };
        peers.push(peer);
    }

    Ok(peers)
}
