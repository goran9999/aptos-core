use crate::{models::aptocracy_models::aptocracy_utils::OrganizationWriteSet, schema::*};
use aptos_api_types::WriteTableItem as APIWriteTableItem;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VoteOptionTableItemDto {
    pub key: String,
    pub value: VoteOptionTableContent,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VoteOptionTableContent {
    pub execution_steps: Vec<ExecutionStepsDto>,
    pub option_elected: bool,
    pub vote_weight: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExecutionStepsDto {
    pub execution_hash: String,
    pub execution_parameters: Vec<String>,
    pub execution_parameter_types: Vec<String>,
    pub executed: bool,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = vote_options)]
pub struct VoteOption {
    pub id: Uuid,
    pub option: String,
    pub vote_weight: i64,
    pub option_elected: bool,
    pub proposal_id: Uuid,
}

impl VoteOption {
    pub fn from(
        vote_option_dto: &VoteOptionTableItemDto,
        treasury_address: String,
        proposal_id: Uuid,
    ) -> Self {
        VoteOption {
            id: Uuid::new_v4(),
            proposal_id,
            option: vote_option_dto.key.clone(),
            vote_weight: vote_option_dto.value.vote_weight.parse::<i64>().unwrap(),
            option_elected: vote_option_dto.value.option_elected,
        }
    }
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = execution_step)]
pub struct ExecutionStep {
    pub id: Uuid,
    pub execution_hash: String,
    pub execution_parameters: String,
    pub execution_paramter_types: String,
    pub executed: bool,
    pub vote_option_id: Uuid,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = vote_record)]
pub struct VoteRecord {
    pub member_address: String,
    pub proposal_id: i64,
    pub treasury_address: String,
    pub voter_weight: i64,
    pub voted_at: NaiveDateTime,
    pub elected_options: Vec<String>,
}

impl ExecutionStep {
    pub fn from(
        execution_step_dto: &ExecutionStepsDto,
        vote_option: Uuid,
        proposal_id: i64,
        treasury_address: String,
    ) -> Self {
        ExecutionStep {
            id: Uuid::new_v4(),
            vote_option_id: vote_option,
            execution_hash: execution_step_dto.execution_hash.clone(),
            execution_parameters: serde_json::to_string(
                &execution_step_dto.execution_parameters.clone(),
            )
            .unwrap(),
            execution_paramter_types: serde_json::to_string(
                &execution_step_dto.execution_parameter_types.clone(),
            )
            .unwrap(),
            executed: execution_step_dto.executed,
        }
    }
}

impl VoteOptionTableContent {
    pub fn from_write_table_item(
        write_table_item: &APIWriteTableItem,
    ) -> Option<VoteOptionTableItemDto> {
        let table_item_data = write_table_item.data.as_ref().unwrap();

        if let Some(OrganizationWriteSet::VoteOptionData(inner)) =
            OrganizationWriteSet::from_table_item_type(
                &table_item_data.value_type.as_str(),
                &table_item_data.value,
            )
            .unwrap()
        {
            println!("Vote option __proccessor {:?}", inner);
            Some(VoteOptionTableItemDto {
                key: table_item_data.key.as_str().unwrap().to_owned(),
                value: inner,
            })
        } else {
            None
        }
    }
}
