mod db;
mod models;

use db::get_peers;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tokio_postgres::{Client, NoTls};
use actix_web::web::Data;
use crate::models::{NetworkType, QueryParams};
use clap::Arg;

// Define a handler function for the "/peer" endpoint
async fn peer_handler(
    query_aprams: web::Query<QueryParams>,
    client: web::Data<Client>,
) -> impl Responder {
    match get_peers(
        NetworkType::from(query_aprams.network.clone()),
        query_aprams.offline_timeout,
        &client,
    )
    .await
    {
        Ok(peers) => HttpResponse::Ok().json(peers),
        Err(e) => {
            eprintln!("Error getting peers: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = clap::App::new("Marci")
        .arg(
            Arg::with_name("db")
                .long("db-url")
                .takes_value(true)
                .required(true)
                .default_value("postgresql://postgres:postgres@127.0.0.1/ckb")
                .help("The URL of the Postgres database"),
        )
        .arg(
            Arg::with_name("bind")
                .long("bind")
                .takes_value(true)
                .required(true)
                .default_value("0.0.0.0:1800")
                .help("The address to bind the server to"),
        )
        .get_matches();

    let db_url = matches.value_of("db_url").unwrap();
    let bind = matches.value_of("bind").unwrap();

    // Read database connection parameters from environment variables
    //let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls)
        .await
        .expect("Failed to connect to database");
    let client = Data::new(client);
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error connecting to database: {}", e);
        }
    });
    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&client))
            .route("/peer", web::get().to(peer_handler))
    })
    .bind(bind)?
    .run()
    .await
}
