use crate::aptocracy::aptocracy::GoverningCollection;
use crate::aptocracy::aptocracy::RoleConfig;

use crate::aptocracy::members::{AptocracyUser, SocialNetwork};
use crate::graphql;
use aptos_indexer::models::aptocracy_models::organization::organization::CollectionInfo;
use aptos_indexer::models::aptocracy_models::organization::organization::{
    GoverningCollectionInfo, RoleConfigData,
};
use aws_sdk_s3::primitives::{ByteStream, SdkBody};
use chrono::NaiveDateTime;
use diesel::sql_types::Uuid;
use juniper::graphql_scalar;
use juniper::{FieldError, FieldResult, GraphQLValue, ScalarValue};
use juniper::{GraphQLScalarValue, ParseScalarResult, Value};
use serde::{Deserialize, Serialize};

pub fn parse_graphql_response<T, K>(res: Result<T, diesel::result::Error>) -> FieldResult<K>
where
    T: AptocracyParser<K>,
{
    match res {
        Ok(data) => FieldResult::Ok(data.from()),
        Err(e) => FieldResult::Err(FieldError::from(e)),
    }
}

pub trait AptocracyParser<T> {
    fn from(self) -> T;
}

pub fn map_to_f64(value: Option<i64>) -> Option<f64> {
    match value {
        Some(data) => Some(data as f64),
        None => None,
    }
}

pub fn map_to_role_config(role_config: String) -> Vec<RoleConfig> {
    let role_config_data = serde_json::from_str::<Vec<RoleConfigData>>(&role_config).unwrap();
    let mut roles: Vec<RoleConfig> = vec![];

    role_config_data.iter().for_each(|item| {
        roles.push(RoleConfig {
            actions: item
                .value
                .org_actions
                .iter()
                .map(|weight| weight.parse::<i32>().unwrap())
                .collect(),
            name: item.key.clone(),
            role_weight: item.value.role_weight.parse::<f64>().unwrap(),
        })
    });

    roles
}

pub fn map_to_governing_collection_info(
    governing_collection_info: String,
) -> Option<GoverningCollection> {
    //TODO: check if this could be done in better way
    if !governing_collection_info.contains("null") {
        let governing_collection_data =
            serde_json::from_str::<CollectionInfo>(&governing_collection_info).unwrap();
        let governing_collection: GoverningCollection = GoverningCollection {
            creator: governing_collection_data.creator,
            name: governing_collection_data.name,
        };
        Some(governing_collection)
    } else {
        None
    }
}

pub fn parse_unix_option(unix_ts: Option<i64>) -> Option<NaiveDateTime> {
    if let Some(timestamp) = unix_ts {
        Some(NaiveDateTime::from_timestamp(timestamp, 0))
    } else {
        None
    }
}

pub fn parse_socials<T>(aptocracy_user: AptocracyUser) -> Vec<T>
where
    T: for<'a> Deserialize<'a>,
{
    if let Some(socials) = aptocracy_user.socials {
        serde_json::from_str::<Vec<T>>(&socials).unwrap()
    } else {
        vec![]
    }
}

pub async fn upload_image_aws(image_base_64: String) -> String {
    let config = aws_config::from_env().load().await;
    let client = aws_sdk_s3::Client::new(&config);

    let bucket = std::env::var("AWS_BUCKET").expect("Failed to load bucket name");

    let decoded = base64::decode(image_base_64).unwrap();
    let image = ByteStream::new(SdkBody::from(decoded));

    let image_key = uuid::Uuid::new_v4().to_string();

    client
        .put_object()
        .bucket(bucket.clone())
        .key(image_key.clone())
        .content_type("image/png")
        .body(image)
        .send()
        .await
        .unwrap();

    let url = format!("https://{}.s3.amazonaws.com/{}", bucket, image_key);

    url
}
