use std::io::Error;
use std::str::FromStr;
use std::vec;

use crate::db;
use actix_web::HttpResponse;
use aptos_indexer::schema::execution_step::{self, dsl::*};
use aptos_indexer::schema::scripts::{self, dsl::*};
use aptos_sdk::crypto::{ed25519, ValidCryptoMaterialStringExt};
use aptos_sdk::move_types::language_storage::{StructTag, TypeTag};
use aptos_sdk::move_types::value::{MoveTypeLayout, MoveValue};
use aptos_sdk::rest_client::Client;
use aptos_sdk::transaction_builder::{TransactionBuilder, TransactionFactory};
use aptos_sdk::types::account_address::AccountAddress;
use aptos_sdk::types::chain_id::ChainId;
use aptos_sdk::types::transaction::{EntryFunction, TransactionArgument};
use aptos_sdk::types::{AccountKey, LocalAccount};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;
#[derive(Deserialize, Serialize)]
pub struct ExecuteScript {
    pub wallet: String,
    pub execution_step_id: String,
    pub transfered_amount: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ScriptArg {
    pub arg_type: ArgType,
    pub arg: String,
}

#[derive(Serialize, Deserialize)]

pub enum ArgType {
    Address,
    Bytes,
    String,
    U8,
    U64,
    U128,
    Bool,
    TypeInfo,
    AddressVector,
}

impl From<&str> for ArgType {
    fn from(value: &str) -> Self {
        match value {
            "address" => Self::Address,
            "u64" => Self::U64,
            "u8" => Self::U8,
            "String" => Self::String,
            "TypeInfo" => Self::TypeInfo,
            "u128" => Self::U128,
            "vector<u8>" => Self::Bytes,
            "bool" => Self::Bool,
            "vector<address>" => Self::AddressVector,
            _ => todo!(),
        }
    }
}

impl ArgType {
    pub fn to_transaction_argument(&self, bytes: Vec<u8>) -> TransactionArgument {
        match self {
            ArgType::Address => {
                TransactionArgument::Address(AccountAddress::from_bytes(bytes).unwrap())
            }
            ArgType::Bool => TransactionArgument::Bool(aptos_sdk::bcs::from_bytes(&bytes).unwrap()),
            ArgType::Bytes => TransactionArgument::U8Vector(bytes),
            ArgType::String => TransactionArgument::U8Vector(bytes),
            ArgType::U128 => TransactionArgument::U128(aptos_sdk::bcs::from_bytes(&bytes).unwrap()),
            ArgType::U64 => TransactionArgument::U64(aptos_sdk::bcs::from_bytes(&bytes).unwrap()),
            ArgType::U8 => TransactionArgument::U8(bytes[0]),
            ArgType::AddressVector => TransactionArgument::U8Vector(bytes),
            _ => todo!(),
        }
    }
}

pub fn get_execution_step(execution_id: String) -> Result<ExecutionStep, diesel::result::Error> {
    let execution_step_id = uuid::Uuid::try_parse(&execution_id).unwrap();
    let conn = &mut db::init_db().get().unwrap();
    execution_step
        .filter(id.eq(execution_step_id))
        .get_result(conn)
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name=scripts)]
pub struct Script {
    pub script_hash: String,
    pub proposal_type: i32,
    pub bytecode: String,
}

pub fn get_execution_script(target_script_hash: &String) -> Result<Script, diesel::result::Error> {
    let conn = &mut db::init_db().get().unwrap();

    scripts
        .filter(script_hash.eq(target_script_hash))
        .get_result(conn)
}

#[derive(Queryable, Clone, Debug)]
#[diesel(belongs_to(VoteOption,foreign_key = vote_option_id))]
#[diesel(table_name = execution_step)]
pub struct ExecutionStep {
    pub id: uuid::Uuid,
    pub execution_hash: String,
    pub execution_parameters: String,
    pub execution_parameter_types: String,
    pub executed: bool,
    pub vote_option_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct ExecuteScriptResponse {
    pub message: String,
    pub execution_step_id: String,
}

pub async fn execute_script(execute_script_dto: ExecuteScript) -> HttpResponse {
    let execution_step_data =
        get_execution_step(execute_script_dto.execution_step_id.clone()).unwrap();
    let script_data = get_execution_script(&execution_step_data.execution_hash).unwrap();

    let mut signer = get_signer().await;

    let tx_builder = execute_script_impl(script_data, execution_step_data)
        .await
        .unwrap();

    let signed_tx = signer.sign_with_transaction_builder(tx_builder);

    let client = get_aptos_client();

    let test = client.submit_and_wait_bcs(&signed_tx).await.unwrap();

    HttpResponse::Ok().json("Script successfully executed")
}

pub async fn get_signer() -> LocalAccount {
    let private_key =
        std::env::var("PRIVATE_KEY").expect("Failed to load custody wallet secret key!");

    let ed25519_key = ed25519::PrivateKey::from_encoded_string(&private_key).unwrap();

    let account_key = AccountKey::from_private_key(ed25519_key);

    let client = get_aptos_client();

    let hex_account_address =
        std::env::var("ACCOUNT_ADDRESS").expect("Failed to load account address");

    let account_address = AccountAddress::from_hex(hex_account_address).unwrap();

    let account = client.get_account(account_address).await.unwrap();

    LocalAccount::new(
        account_address,
        account_key,
        account.inner().sequence_number,
    )
}

pub fn get_aptos_client() -> Client {
    let rpc_url = std::env::var("RPC_URL").expect("Failed to get rpc url");

    Client::new(Url::parse(&rpc_url).unwrap())
}

pub async fn execute_script_impl(
    script: Script,
    execution_step_data: ExecutionStep,
) -> Result<TransactionBuilder, ()> {
    let signer = get_signer().await;

    let parsed_args: Vec<String> =
        serde_json::from_str(&execution_step_data.execution_parameters).unwrap();

    let mut serialized_args: Vec<Vec<u8>> = vec![];

    for parsed_arg in parsed_args.iter() {
        let decoded = hex::decode(&parsed_arg[2..]).unwrap();

        serialized_args.push(decoded);
    }

    println!("SERIALIZED {:?}", serialized_args);

    parse_script_args(serialized_args, execution_step_data, script, parsed_args)
}

pub fn parse_script_args(
    serialized_args: Vec<Vec<u8>>,
    execution_step_data: ExecutionStep,
    script_data: Script,
    plain_args: Vec<String>,
) -> Result<TransactionBuilder, ()> {
    println!(
        "PARSED PAR TYPES  EXE {:?}",
        execution_step_data.execution_parameter_types
    );
    let parsed_parameter_types =
        serde_json::from_str::<Vec<&str>>(&execution_step_data.execution_parameter_types).unwrap();
    if serialized_args.len() != parsed_parameter_types.len() {
        return Err(());
    };

    let mut transaction_arguments: Vec<TransactionArgument> = vec![];

    println!("PARSED PAR TYPES {:?}", parsed_parameter_types);

    let mut type_arg: Vec<TypeTag> = vec![];

    for (index, serialized_arg) in serialized_args.iter().enumerate() {
        let arg_type = ArgType::from(*parsed_parameter_types.get(index).unwrap());

        match arg_type {
            ArgType::TypeInfo => {
                let parsed =
                    String::from_utf8(hex::decode(&plain_args.get(index).unwrap()[2..]).unwrap())
                        .unwrap();

                type_arg.push(TypeTag::from_str("0x1::aptos_coin::AptosCoin").unwrap());
            }
            _ => transaction_arguments
                .push(arg_type.to_transaction_argument(serialized_arg.to_vec())),
        }
    }

    let parsed_script_bytecode: Vec<u8> = serde_json::from_str(&script_data.bytecode).unwrap();

    let tx_factory = TransactionFactory::new(ChainId::from_str("63").unwrap());

    println!("ARG__TEST {:?}", type_arg);
    println!("TYPE ARG__TEST {:?}", transaction_arguments);

    let tx_builder = tx_factory.script(aptos_sdk::types::transaction::Script::new(
        parsed_script_bytecode,
        type_arg,
        transaction_arguments,
    ));

    Ok(tx_builder)
}
