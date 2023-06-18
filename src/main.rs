mod db;
mod models;

use crate::models::{NetworkType, QueryParams};
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{http, web, App, HttpResponse, HttpServer, Responder};
use clap::Arg;
use db::get_peers;
use tokio_postgres::{Client, NoTls};

// Define a handler function for the "/peer" endpoint
async fn peer_handler(
    query_params: web::Query<QueryParams>,
    client: web::Data<Client>,
) -> impl Responder {
    match get_peers(
        NetworkType::from(query_params.network.clone()),
        query_params.offline_timeout,
        query_params.unknown_offline_timeout,
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
                .required(false)
                .default_value("0.0.0.0:1800")
                .help("The address to bind the server to"),
        )
        .get_matches();

    let db_url = matches.value_of("db").unwrap();
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
    let app = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(Data::clone(&client))
            .route("/peer", web::get().to(peer_handler))
            .route("/", web::get().to(peer_handler))
    })
    .bind(bind)?;

    app.run().await
}
