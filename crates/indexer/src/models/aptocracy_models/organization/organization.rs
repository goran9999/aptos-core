use crate::{
    models::aptocracy_models::aptocracy_utils::{
        parse_move_option, MoveOption, OrganizationResource, TypeDef,
    },
    processors::aptocracy_processor::AptocracyProcessor,
    schema::governance,
    schema::organization,
};
use aptos_api_types::deserialize_from_string;
use aptos_api_types::{Transaction as APITransaction, WriteSetChange as APIWriteSetChange};
use chrono::Utc;
use clap::builder::TypedValueParser;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]

pub struct OrganizationDto {
    pub creator: String,
    pub default_role: String,
    pub governing_coin: GoverningCoin,
    pub governing_collection_info: GoverningCollectionInfo,
    pub invite_only: bool,
    pub main_governance: MoveOption<String>,
    pub max_voter_weight: MoveOption<String>,
    pub name: String,
    pub org_type: String,
    pub organization_metadata: OrganizationMetadata,
    pub role_config: RoleConfig,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct GoverningCoin {
    pub vec: Vec<TypeDef>,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct GoverningCollectionInfo {
    vec: Vec<CollectionInfo>,
}
#[derive(Serialize, Deserialize, Debug)]

pub struct CollectionInfo {
    pub creator: String,
    pub name: String,
}
#[derive(Serialize, Deserialize, Debug)]

pub struct OrganizationMetadata {
    pub treasury_count: u64,
    pub main_treasury: MoveOption<String>,
}
#[derive(Serialize, Deserialize, Debug)]

pub struct RoleConfig {
    data: Vec<RoleConfigData>,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct RoleConfigData {
    pub key: String,
    pub value: RoleConfigValues,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct RoleConfigValues {
    pub org_actions: Vec<String>,
    pub role_weight: String,
}

impl OrganizationDto {
    pub fn from(self, address: String) -> Organization {
        Organization {
            address,
            name: self.name,
            creator: self.creator,
            default_role: self.default_role,
            governing_coin: serde_json::to_string(&self.governing_coin.vec.get(0)).unwrap(),
            governing_collection_info: serde_json::to_string(
                &self.governing_collection_info.vec.get(0),
            )
            .unwrap(),
            invite_only: self.invite_only,
            main_governance: parse_move_option(self.main_governance),
            max_voter_weight: parse_move_option(self.max_voter_weight),
            org_type: self.org_type,
            treasury_count: self
                .organization_metadata
                .treasury_count
                .try_into()
                .unwrap(),
            role_config: serde_json::to_string(&self.role_config.data).unwrap(),
            created_at: Utc::now().naive_utc(),
            main_treasury: self.organization_metadata.main_treasury.vec.get(0).cloned(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Insertable, AsChangeset)]
#[diesel(table_name = organization)]
pub struct Organization {
    pub address: String,
    pub name: String,
    pub creator: String,
    pub default_role: String,
    pub governing_coin: String,
    pub governing_collection_info: String,
    pub invite_only: bool,
    pub main_governance: Option<i64>,
    pub max_voter_weight: Option<i64>,
    pub org_type: String,
    pub treasury_count: i32,
    pub role_config: String,
    pub created_at: chrono::NaiveDateTime,
    pub main_treasury: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Insertable, AsChangeset)]
#[diesel(table_name = governance)]
pub struct Governance {
    pub aptocracy_address: String,
    pub governance_id: i64,
    pub max_voting_time: i64,
    pub quorum: i64,
    pub approval_quorum: i64,
    pub early_tipping: bool,
    pub valid_from: i64,
    pub valid_to: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GovernancesDto {
    pub governances: GovernancesStruct,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GovernancesStruct {
    pub data: Vec<GovernanceData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GovernanceData {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub key: i64,
    pub value: GovernanceDto,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GovernanceDto {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_voting_time: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub approval_quorum: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub quorum: i64,
    pub early_tipping: bool,
    pub governance_metadata: GovernanceMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GovernanceMetadata {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub valid_from: i64,
    pub valid_to: MoveOption<String>,
    pub aptocracy_address: String,
}

impl Governance {
    pub fn get_governance_resource() -> String {
        let module_address = AptocracyProcessor::get_module_address();
        let governance_module = format!("{}::organization:Governances", module_address);

        governance_module
    }
}

impl Organization {
    pub fn from_transaction(transaction: &APITransaction) -> (Vec<Organization>, Vec<Governance>) {
        let mut orgs: Vec<Organization> = vec![];
        let mut governances: Vec<Governance> = vec![];

        if let APITransaction::UserTransaction(user_txn) = transaction {
            for wsc in user_txn.info.changes.iter() {
                if let APIWriteSetChange::WriteResource(write_resource) = wsc {
                    if let Some(OrganizationResource::CreateOrganization(inner)) =
                        OrganizationResource::from_write_resource(
                            &write_resource,
                            user_txn.info.version.0 as i64,
                        )
                        .unwrap()
                    {
                        orgs.push(inner.from(write_resource.address.to_string()));
                    }

                    if let Some(OrganizationResource::CreateGovernance(inner)) =
                        OrganizationResource::from_write_resource(
                            &write_resource,
                            user_txn.info.version.0 as i64,
                        )
                        .unwrap()
                    {
                        for governance in inner.governances.data.into_iter() {
                            governances.push(Governance {
                                aptocracy_address: governance
                                    .value
                                    .governance_metadata
                                    .aptocracy_address,
                                governance_id: governance.key,
                                max_voting_time: governance.value.max_voting_time,
                                quorum: governance.value.quorum,
                                approval_quorum: governance.value.approval_quorum,
                                early_tipping: governance.value.early_tipping,
                                valid_from: governance.value.governance_metadata.valid_from,
                                valid_to: parse_move_option(
                                    governance.value.governance_metadata.valid_to,
                                ),
                            })
                        }
                    }
                }
            }
        }
        (orgs, governances)
    }

    pub fn get_organization_resource() -> String {
        let module_address = AptocracyProcessor::get_module_address();

        let organization = format!("{}::organization::Organization", module_address);

        organization
    }
}
