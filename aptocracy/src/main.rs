pub mod aptocracy;
pub mod db;
pub mod helpers;
pub mod proposals;
pub mod treasury;
use std::sync::Arc;

use crate::{graphql::create_schema, routes::init_routes};

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use db::init_db;
pub mod graphql;
pub mod routes;
extern crate dotenv;
#[actix_rt::main]
async fn main() {
    dotenv::dotenv().ok();

    let schema = Arc::new(create_schema());
    let db = init_db();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method(),
            )
            .data(db.clone())
            .data(schema.clone())
            .configure(init_routes)
    });

    let host = std::env::var("HOST").expect("Missing host in env");
    let port = std::env::var("PORT").expect("Missing port in env");
    let address = format!("{}:{}", host, port);

    server.bind(address).unwrap().run().await.unwrap();
}
