use super::super::aptocracy_utils::{parse_move_option, parse_move_string, MoveOption, MoveTable};
use super::vote_options::{VoteOptionTableContent, VoteOptionTableItemDto};
use crate::models::aptocracy_models::aptocracy_utils::{AptocracyEvent, OrganizationWriteSet};
use crate::schema::*;
use aptos_api_types::{
    Transaction as APITransaction, WriteSetChange as APIWriteSetChange,
    WriteTableItem as APIWriteTableItem,
};
use diesel::prelude::*;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = proposal)]
pub struct Proposal {
    pub id: uuid::Uuid,
    pub aptocracy_address: String,
    pub treasury_address: String,
    pub proposal_id: i64,
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
    pub proposal_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProposalDto {
    pub cancelled_at: MoveOption<String>,
    pub created_at: String,
    pub creator: String,
    pub description: String,
    pub early_tipping: bool,
    pub executed_at: MoveOption<String>,
    pub max_vote_weight: String,
    pub max_voter_options: String,
    pub max_voting_time: String,
    pub name: String,
    pub proposal_content: ProposalMetadata,
    pub state: i32,
    pub vote_options: MoveTable,
    pub vote_threshold: VoteThreshold,
    pub voting_finalized_at: MoveOption<String>,
}

#[repr(u8)]
#[derive(Deserialize, Serialize, Debug, TryFromPrimitive)]

pub enum ProposalState {
    Voting = 0,
    Succeded = 1,
    Executing = 2,
    Completed = 3,
    Canceled = 4,
    Defeated = 5,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProposalMetadata {
    pub discussion_link: String,
    pub treasury_address: String,
    pub aptocracy_address: String,
    pub number_of_votes: String,
    pub proposal_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoteThreshold {
    pub approval_quorum: String,
    pub quorum: String,
}

impl Proposal {
    pub fn from_transaction(
        transaction: &APITransaction,
    ) -> (
        Vec<Proposal>,
        Vec<VoteOptionTableItemDto>,
        Vec<AptocracyEvent>,
    ) {
        let mut proposals: Vec<Proposal> = vec![];
        let mut proposal_options: Vec<VoteOptionTableItemDto> = vec![];
        let mut proposal_events: Vec<AptocracyEvent> = vec![];

        if let APITransaction::UserTransaction(user_tnx) = transaction {
            for wsc in user_tnx.info.changes.iter() {
                let maybe_proposal_data =
                    if let APIWriteSetChange::WriteTableItem(write_table_item) = &wsc {
                        Proposal::from_write_table_item(write_table_item)
                    } else {
                        None
                    };

                let maybe_vote_option =
                    if let APIWriteSetChange::WriteTableItem(write_table_item) = &wsc {
                        VoteOptionTableContent::from_write_table_item(write_table_item)
                    } else {
                        None
                    };

                if let Some(vote_option) = maybe_vote_option {
                    println!("VOTE OPTION __processor, ${:?}", vote_option);
                    proposal_options.push(vote_option);
                }
                if let Some(proposal_data) = maybe_proposal_data {
                    proposals.push(proposal_data);
                }
            }

            for event in user_tnx.events.iter() {
                let maybe_aptocracy_event =
                    AptocracyEvent::from_event(event.typ.to_string().as_str(), &event.data);
                if let Some(aptocracy_event) = maybe_aptocracy_event {
                    proposal_events.push(aptocracy_event);
                }
            }
        }

        (proposals, proposal_options, proposal_events)
    }

    pub fn from_write_table_item(write_table_item: &APIWriteTableItem) -> Option<Proposal> {
        let table_item_data = write_table_item.data.as_ref().unwrap();

        if let Some(OrganizationWriteSet::ProposalData(inner)) =
            OrganizationWriteSet::from_table_item_type(
                &table_item_data.value_type.as_str(),
                &table_item_data.value,
            )
            .unwrap()
        {
            Some(inner.from_dto(&table_item_data.key.to_string()))
        } else {
            None
        }
    }
}

impl ProposalDto {
    pub fn from_dto(self, proposal_id: &str) -> Proposal {
        Proposal {
            id: Uuid::new_v4(),
            aptocracy_address: self.proposal_content.aptocracy_address.clone(),
            treasury_address: self.proposal_content.treasury_address.clone(),
            proposal_id: parse_move_string(proposal_id).parse::<i64>().unwrap(),
            name: self.name.clone(),
            description: self.description.clone(),
            discussion_link: self.proposal_content.discussion_link.clone(),
            creator: self.creator.clone(),
            max_vote_weight: self.max_vote_weight.parse::<i64>().unwrap(),
            cancelled_at: parse_move_option(self.cancelled_at.clone()),
            created_at: self.created_at.parse::<i64>().unwrap(),
            early_tipping: self.early_tipping,
            executed_at: parse_move_option(self.executed_at.clone()),
            max_voter_options: self.max_voter_options.parse::<i64>().unwrap(),
            max_voting_time: self.max_voting_time.parse::<i64>().unwrap(),
            state: self.state,
            vote_threshold: serde_json::to_string(&self.vote_threshold).unwrap(),
            voting_finalized_at: parse_move_option(self.voting_finalized_at.clone()),
            proposal_type: self.proposal_content.proposal_type.clone(),
        }
    }
}
