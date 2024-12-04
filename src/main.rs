extern crate diesel_migrations;
extern crate diesel;
extern crate actix_web;

use actix_web::{App, HttpServer, web::Data};
use dotenv::dotenv;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::pg::PgConnection;
use std::env;

mod routes;
mod schema;
mod transaction;
mod setup;
mod auth;
mod users;
mod integration_tests;

pub type DBPool = Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load `.env` file with environment variables if present
    dotenv().ok();

    let api_port = env::var("API_PORT").unwrap_or_else(|_| "8080".to_string());

    let database_url = env::var("DB_CONNECTION_URL").expect("DB_CONNECTION_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager).expect("Failed to create pool.");

    let mut conn = pool.get().expect("Failed to get connection from pool.");
    setup::run_migrations(&mut conn);

    HttpServer::new(move || {
        App::new()
            // Set up DB pool to be used with web::Data<Pool> extractor
            .app_data(Data::new(pool.clone()))
            // register HTTP requests handlers
            .service(auth::authenticate)
            .service(routes::lime_eth_transactions_hashes)
            .service(routes::lime_eth_rlphex)
            .service(routes::lime_all)
            .service(routes::lime_my)
    })
    .bind(format!("0.0.0.0:{}", api_port))?
    .run()
    .await
}
