use std::vec;

use aptos_indexer::{
    models::aptocracy_models::organization,
    schema::{
        execution_step::{self, dsl::*},
        proposal::{self, dsl::*},
        vote_options,
        vote_record::{self, dsl::*},
    },
};
use chrono::NaiveDateTime;
use diesel::associations::{BelongsTo, HasTable};
use diesel::prelude::*;
use juniper::{FieldResult, GraphQLObject};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::helpers::{map_to_f64, parse_graphql_response, AptocracyParser};
#[derive(Queryable, Debug, Identifiable, Clone)]
#[diesel(table_name = proposal)]
pub struct Proposal {
    pub id: Uuid,
    pub proposal_id: i64,
    pub treasury_address: String,
    pub aptocracy_address: String,
    pub name: String,
    pub description: String,
    pub discussion_link: String,
    pub creator: String,
    pub max_vote_weight: i64,
    pub cancelled_at: Option<i64>,
    pub created_at: i64,
    pub early_tipping: bool,
    pub executed_at: Option<i64>,
    pub max_voter_options: i64,
    pub max_voting_time: i64,
    pub state: i32,
    pub vote_threshold: String,
    pub voting_finalized_at: Option<i64>,
    pub proposal_type: Option<String>,
}

#[derive(Queryable, Debug, Clone, Associations, Identifiable)]
#[diesel(belongs_to(Proposal,foreign_key = proposal_id))]
#[diesel(table_name = vote_options)]
pub struct VoteOption {
    pub id: Uuid,
    pub option: String,
    pub vote_weight: i64,
    pub option_elected: bool,
    pub proposal_id: Uuid,
}

#[derive(Queryable, Debug, Clone, Associations, Identifiable)]
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

const TREASURY_PROPOSALS: &[&str] = &["Discussion", "Transfer", "Withdrawal", "Custom"];
const APTOCRACY_PROPOSALS: &[&str] = &[
    "UpdateMainGovernance",
    "UpdateMainTreasury",
    "ChangeGovernance",
];

impl Proposal {
    pub fn get_proposals_for_treasury(
        conn: &mut PgConnection,
        wanted_treasury_address: String,
    ) -> FieldResult<Vec<ProposalDto>> {
        let mut proposal_dtos: Vec<ProposalDto> = vec![];

        let proposals = proposal::table
            .filter(proposal::treasury_address.eq(wanted_treasury_address))
            .filter(proposal::proposal_type.eq_any(TREASURY_PROPOSALS))
            .order_by(proposal::created_at.desc())
            .load::<Proposal>(conn)
            .unwrap();

        let vote_options = VoteOption::belonging_to(&proposals)
            .load::<VoteOption>(conn)
            .unwrap();

        let execution_steps_data = ExecutionStep::belonging_to(&vote_options)
            .load::<ExecutionStep>(conn)
            .unwrap();

        for proposal_data in proposals.iter() {
            let vote_records =
                vote_record
                    .filter(vote_record::proposal_id.eq(proposal_data.proposal_id).and(
                        vote_record::treasury_address.eq(proposal_data.treasury_address.clone()),
                    ))
                    .load::<VoteRecord>(conn)
                    .unwrap()
                    .from();
            let related_vote_options = vote_options
                .clone()
                .into_iter()
                .filter(|vote_option| vote_option.proposal_id == proposal_data.id)
                .collect::<Vec<VoteOption>>();

            let mut vote_options_dto: Vec<VoteOptionDto> = vec![];

            for related_vote_option in related_vote_options.iter() {
                let related_execution_steps = execution_steps_data
                    .clone()
                    .into_iter()
                    .filter(|step| step.vote_option_id == related_vote_option.id)
                    .collect::<Vec<ExecutionStep>>()
                    .from();

                vote_options_dto.push(related_vote_option.to_dto(related_execution_steps));
            }

            proposal_dtos.push(proposal_data.clone().from(vote_options_dto, vote_records));
        }

        FieldResult::Ok(proposal_dtos)
    }

    pub fn get_signle_proposal_for_treasury(
        conn: &mut PgConnection,
        wanted_treasury_address: String,
        wanted_proposal_id: i64,
    ) -> FieldResult<ProposalDto> {
        let proposal_data = proposal
            .filter(proposal::proposal_id.eq(wanted_proposal_id))
            .filter(proposal::proposal_type.eq_any(TREASURY_PROPOSALS))
            .filter(proposal::treasury_address.eq(wanted_treasury_address))
            .order_by(proposal::created_at.desc())
            .get_result::<Proposal>(conn)
            .unwrap();

        let vote_options = VoteOption::belonging_to(&proposal_data)
            .load::<VoteOption>(conn)
            .unwrap();

        let execution_steps_data = ExecutionStep::belonging_to(&vote_options)
            .load::<ExecutionStep>(conn)
            .unwrap();

        let vote_records = vote_record
            .filter(vote_record::proposal_id.eq(proposal_data.proposal_id))
            .filter(vote_record::treasury_address.eq(proposal_data.treasury_address.clone()))
            .load::<VoteRecord>(conn)
            .unwrap()
            .from();

        let mut vote_options_dto: Vec<VoteOptionDto> = vec![];

        for vote_option in vote_options.iter() {
            let related_execution_steps = execution_steps_data
                .clone()
                .into_iter()
                .filter(|item| item.vote_option_id == vote_option.id)
                .collect::<Vec<ExecutionStep>>()
                .from();

            vote_options_dto.push(vote_option.to_dto(related_execution_steps));
        }

        FieldResult::Ok(proposal_data.clone().from(vote_options_dto, vote_records))
    }

    pub fn get_proposals_for_aptocracy(
        conn: &mut PgConnection,
        wanted_aptocracy_address: String,
    ) -> FieldResult<Vec<ProposalDto>> {
        let mut proposal_dtos: Vec<ProposalDto> = vec![];

        let proposals = proposal::table
            .filter(proposal::aptocracy_address.eq(wanted_aptocracy_address))
            .filter(proposal::proposal_type.eq_any(APTOCRACY_PROPOSALS))
            .order_by(proposal::created_at.desc())
            .load::<Proposal>(conn)
            .unwrap();

        let vote_options = VoteOption::belonging_to(&proposals)
            .load::<VoteOption>(conn)
            .unwrap();

        let execution_steps_data = ExecutionStep::belonging_to(&vote_options)
            .load::<ExecutionStep>(conn)
            .unwrap();

        for proposal_data in proposals.iter() {
            let vote_records =
                vote_record
                    .filter(vote_record::proposal_id.eq(proposal_data.proposal_id).and(
                        vote_record::treasury_address.eq(proposal_data.treasury_address.clone()),
                    ))
                    .load::<VoteRecord>(conn)
                    .unwrap()
                    .from();
            let related_vote_options = vote_options
                .clone()
                .into_iter()
                .filter(|vote_option| vote_option.proposal_id == proposal_data.id)
                .collect::<Vec<VoteOption>>();

            let mut vote_options_dto: Vec<VoteOptionDto> = vec![];

            for related_vote_option in related_vote_options.iter() {
                let related_execution_steps = execution_steps_data
                    .clone()
                    .into_iter()
                    .filter(|step| step.vote_option_id == related_vote_option.id)
                    .collect::<Vec<ExecutionStep>>()
                    .from();

                vote_options_dto.push(related_vote_option.to_dto(related_execution_steps));
            }

            proposal_dtos.push(proposal_data.clone().from(vote_options_dto, vote_records));
        }

        FieldResult::Ok(proposal_dtos)
    }

    pub fn get_signle_proposal_for_aptocracy(
        conn: &mut PgConnection,
        wanted_aptocracy_address: String,
        wanted_proposal_id: i64,
    ) -> FieldResult<ProposalDto> {
        let proposal_data = proposal
            .filter(proposal::proposal_id.eq(wanted_proposal_id))
            .filter(proposal::aptocracy_address.eq(wanted_aptocracy_address))
            .get_result::<Proposal>(conn)
            .unwrap();

        let vote_options = VoteOption::belonging_to(&proposal_data)
            .load::<VoteOption>(conn)
            .unwrap();

        let execution_steps_data = ExecutionStep::belonging_to(&vote_options)
            .load::<ExecutionStep>(conn)
            .unwrap();

        let vote_records = vote_record
            .filter(vote_record::proposal_id.eq(proposal_data.proposal_id))
            .filter(vote_record::treasury_address.eq(proposal_data.treasury_address.clone()))
            .load::<VoteRecord>(conn)
            .unwrap()
            .from();

        let mut vote_options_dto: Vec<VoteOptionDto> = vec![];

        for vote_option in vote_options.iter() {
            let related_execution_steps = execution_steps_data
                .clone()
                .into_iter()
                .filter(|item| item.vote_option_id == vote_option.id)
                .collect::<Vec<ExecutionStep>>()
                .from();

            vote_options_dto.push(vote_option.to_dto(related_execution_steps));
        }

        FieldResult::Ok(proposal_data.clone().from(vote_options_dto, vote_records))
    }

    pub fn get_latest_proposal_in_aptocracy(
        conn: &mut PgConnection,
        wanted_aptocracy_address: String,
    ) -> FieldResult<ProposalDto> {
        let proposal_data = proposal
            .filter(proposal::aptocracy_address.eq(wanted_aptocracy_address))
            .order_by(proposal::proposal_id.desc())
            .limit(1)
            .get_result::<Proposal>(conn)
            .unwrap();

        let vote_options = VoteOption::belonging_to(&proposal_data)
            .load::<VoteOption>(conn)
            .unwrap();

        let execution_steps_data = ExecutionStep::belonging_to(&vote_options)
            .load::<ExecutionStep>(conn)
            .unwrap();

        let vote_records = vote_record
            .filter(vote_record::proposal_id.eq(proposal_data.proposal_id))
            .filter(vote_record::treasury_address.eq(proposal_data.treasury_address.clone()))
            .load::<VoteRecord>(conn)
            .unwrap()
            .from();

        let mut vote_options_dto: Vec<VoteOptionDto> = vec![];

        for vote_option in vote_options.iter() {
            let related_execution_steps = execution_steps_data
                .clone()
                .into_iter()
                .filter(|item| item.vote_option_id == vote_option.id)
                .collect::<Vec<ExecutionStep>>()
                .from();

            vote_options_dto.push(vote_option.to_dto(related_execution_steps));
        }

        FieldResult::Ok(proposal_data.clone().from(vote_options_dto, vote_records))
    }
}

#[derive(GraphQLObject)]
pub struct ProposalDto {
    pub proposal_id: f64,
    pub treasury_address: String,
    pub aptocracy_address: String,
    pub name: String,
    pub description: String,
    pub discussion_link: String,
    pub creator: String,
    pub max_vote_weight: f64,
    pub cancelled_at: Option<f64>,
    pub created_at: f64,
    pub early_tipping: bool,
    pub executed_at: Option<f64>,
    pub max_voter_options: f64,
    pub max_voting_time: f64,
    pub state: i32,
    pub vote_threshold: String,
    pub voting_finalized_at: Option<f64>,
    pub vote_options: Vec<VoteOptionDto>,
    pub vote_records: Vec<VoteRecordDto>,
    pub proposal_type: String,
}

#[derive(GraphQLObject)]
pub struct VoteOptionDto {
    pub option: String,
    pub vote_weight: f64,
    pub option_elected: bool,
    pub execution_steps: Vec<ExecutionStepsDto>,
}

#[derive(GraphQLObject)]
pub struct ExecutionStepsDto {
    pub id: String,
    pub execution_hash: String,
    pub execution_parameters: String,
    pub execution_parameter_types: String,
    pub executed: bool,
}

#[derive(Queryable, Debug, Associations, Clone)]
#[diesel(belongs_to(Proposal))]
#[diesel(table_name=vote_record)]
pub struct VoteRecord {
    pub member_address: String,
    pub proposal_id: i64,
    pub treasury_address: String,
    pub voter_weight: i64,
    pub elected_options: Vec<Option<String>>,
    pub voted_at: NaiveDateTime,
}

#[derive(GraphQLObject)]
pub struct VoteRecordDto {
    pub member_address: String,
    pub proposal_id: f64,
    pub treasury_address: String,
    pub voter_weight: f64,
    pub elected_options: Vec<Option<String>>,
    pub voted_at: NaiveDateTime,
}

impl AptocracyParser<VoteRecordDto> for VoteRecord {
    fn from(self) -> VoteRecordDto {
        VoteRecordDto {
            member_address: self.member_address,
            proposal_id: self.proposal_id as f64,
            treasury_address: self.treasury_address,
            voter_weight: self.voter_weight as f64,
            elected_options: self.elected_options,
            voted_at: self.voted_at,
        }
    }
}

impl AptocracyParser<ExecutionStepsDto> for ExecutionStep {
    fn from(self) -> ExecutionStepsDto {
        ExecutionStepsDto {
            execution_hash: self.execution_hash,
            execution_parameters: self.execution_parameters,
            execution_parameter_types: self.execution_parameter_types,
            executed: self.executed,
            id: self.id.to_string(),
        }
    }
}

impl VoteOption {
    fn to_dto(&self, steps: Vec<ExecutionStepsDto>) -> VoteOptionDto {
        VoteOptionDto {
            option: self.option.clone(),
            vote_weight: self.vote_weight as f64,
            option_elected: self.option_elected,
            execution_steps: steps,
        }
    }
}

impl AptocracyParser<Vec<VoteRecordDto>> for Vec<VoteRecord> {
    fn from(self) -> Vec<VoteRecordDto> {
        self.iter().map(|item| item.clone().from()).collect()
    }
}

impl AptocracyParser<Vec<ExecutionStepsDto>> for Vec<ExecutionStep> {
    fn from(self) -> Vec<ExecutionStepsDto> {
        self.iter().map(|item| item.clone().from()).collect()
    }
}

impl Proposal {
    fn from(
        self,
        vote_options: Vec<VoteOptionDto>,
        vote_records: Vec<VoteRecordDto>,
    ) -> ProposalDto {
        ProposalDto {
            proposal_id: self.proposal_id as f64,
            treasury_address: self.treasury_address,
            aptocracy_address: self.aptocracy_address,
            name: self.name,
            description: self.description,
            discussion_link: self.discussion_link,
            creator: self.creator,
            max_vote_weight: self.max_vote_weight as f64,
            cancelled_at: map_to_f64(self.cancelled_at),
            created_at: self.created_at as f64,
            early_tipping: self.early_tipping,
            executed_at: map_to_f64(self.executed_at),
            max_voter_options: self.max_voter_options as f64,
            max_voting_time: self.max_voting_time as f64,
            state: self.state,
            vote_threshold: self.vote_threshold,
            voting_finalized_at: map_to_f64(self.voting_finalized_at),
            vote_options,
            vote_records,
            proposal_type: self.proposal_type.unwrap(),
        }
    }
}
