use crate::{
    db,
    handlers::{execute_script, ExecuteScript, Script},
};
use actix_web::{get, post, web, HttpResponse};
use aptos_indexer::schema::scripts::{self, dsl::*};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[post("/execute")]
pub async fn execute_script_handler(data: web::Json<ExecuteScript>) -> HttpResponse {
    execute_script(data.0).await
}

pub fn bind_roures(config: &mut web::ServiceConfig) {
    config
        .service(execute_script_handler)
        .service(store_script_content)
        .service(add_new_script);
}

#[post("/store-script")]
pub async fn store_script_content(script_data: web::Json<StoreScriptData>) -> HttpResponse {
    let conn = &mut db::init_db();

    let mut script_file =
        serde_json::to_string(&fs::read(&script_data.0.script_path_data).unwrap()).unwrap();

    diesel::insert_into(scripts::table)
        .values(&Script {
            bytecode: script_file,
            proposal_type: script_data.proposal_type,
            script_hash: script_data.script_hash_data.clone(),
        })
        .execute(&mut conn.get().unwrap());

    HttpResponse::Ok().json("Stored script")
}

#[post("/add-script")]
pub async fn add_new_script(script_dto: web::Json<ScriptDto>) -> HttpResponse {
    let conn = &mut db::init_db();

    diesel::insert_into(scripts::table)
        .values(&Script {
            bytecode: script_dto.script_bytecode_data.clone(),
            proposal_type: script_dto.proposal_type,
            script_hash: script_dto.script_hash_data.clone(),
        })
        .execute(&mut conn.get().unwrap());

    HttpResponse::Ok().json("Script added")
}

#[derive(Serialize, Deserialize)]
pub struct StoreScriptData {
    pub script_hash_data: String,
    pub script_path_data: String,
    pub proposal_type: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ScriptDto {
    pub script_hash_data: String,
    pub script_bytecode_data: String,
    pub proposal_type: i32,
}
