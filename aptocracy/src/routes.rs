use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse};
use juniper::{
    http::{playground::playground_source, GraphQLRequest},
    FieldResult,
};

use crate::{
    db::PgPool,
    graphql::{AptocracySchema, GraphQlContext},
};
#[get("/graphql")]
pub async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source("/graphql", None))
}

#[post("/graphql")]
pub async fn aptocracy_handler(
    pool: web::Data<PgPool>,
    data: web::Json<GraphQLRequest>,
    schema: web::Data<Arc<AptocracySchema>>,
) -> HttpResponse {
    let context = GraphQlContext {
        pool: pool.get_ref().to_owned(),
    };

    let response = data.execute(&schema, &context).await;

    HttpResponse::Ok().json(response)
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config
        .service(graphql_playground)
        .service(aptocracy_handler);
}
