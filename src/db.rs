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

    let query = format!("SELECT
    DISTINCT(peer.address),
    peer.id,
    peer.ip,
    peer.version,
    peer.time as last_seen,
    ipinfo.country,
    ipinfo.city
FROM {}.peer
LEFT JOIN {}.ipinfo AS ipinfo ON peer.ip = ipinfo.ip
WHERE peer.time > (now() - interval '3 months')
ORDER BY peer.address, peer.id", main_scheme, main_scheme);

    let rows = client.query(query.as_str(), &[]).await?;
    let mut peers = Vec::new();

    for row in rows {
        let last_seen: SystemTime = row.get(4);
        if last_seen.elapsed().unwrap() > Duration::from_secs(offline_min * 60) {
            continue;
        }

        let version: String = row.get(3);
        let version_short = if version.is_empty() { String::new() } else { Regex::new(r"^(.*?)[^0-9.].*$").unwrap().captures(&version).unwrap()[1].to_owned() };
        let city = row.get::<_, Option<String>>(6).unwrap_or_default();
        let country = row.get::<_, Option<String>>(5).unwrap_or_default();

        let geolocation_query = format!("
                                        SELECT
                                            latitude,
                                            longitude
                                        FROM common_info.ip_info
                                        WHERE city = '{}' OR state1 = '{}'
                                        LIMIT 1", city, city);

        let geolocation_rows = client.query(geolocation_query.as_str(), &[]).await?;
        let geolocation_row = geolocation_rows.first();

        let latitude: Option<f64> = geolocation_row.map(|row| row.get::<_, Option<Decimal>>(0).unwrap_or_default().to_f64().unwrap());
        let longitude: Option<f64> = geolocation_row.map(|row| row.get::<_, Option<Decimal>>(1).unwrap_or_default().to_f64().unwrap());

        let peer = Peer {
            id: row.get(1),
            ip: row.get(2),
            version,
            version_short,
            last_seen: Some(last_seen),
            address: row.get(0),
            country: Some(country),
            city: Some(city),
            latitude: latitude,
            longitude: longitude,
        };
        peers.push(peer);
    }
    Ok(peers)
}
