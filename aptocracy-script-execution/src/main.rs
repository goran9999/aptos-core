pub mod db;
pub mod handlers;
pub mod routes;
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use db::init_db;
use routes::bind_roures;
extern crate dotenv;

#[actix_rt::main]
async fn main() {
    dotenv::dotenv().ok();

    let db = init_db();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method(),
            )
            .configure(bind_roures)
            .data(db.clone())
    });

    let host = std::env::var("HOST").expect("Failed to load host");
    let port = std::env::var("PORT").expect("Failed to load port");

    let endpoint = format!("{}:{}", host, port);

    server.bind(endpoint).unwrap().run().await.unwrap();
}
