use super::{organization::organization::{ OrganizationDto, GovernancesDto}, proposal::{proposals::{ProposalMetadata, ProposalDto}, vote_options::{VoteOptionTableItemDto, VoteOptionTableContent}}, treasury::treasury::{TreasuryDto, TreasuryMetadata}};
use crate::{models::{move_resources::MoveResource, aptocracy_models::{treasury::treasury::Treasury, organization::organization::{Organization, Governance}}}};
use anyhow::{Context, Result};
use aptos_api_types::WriteResource;
use serde::{Deserialize, Serialize};





pub fn parse_move_option(mvw: MoveOption<String>) -> Option<i64> {
    if let Some(value) = mvw.vec.get(0) {
        Some(value.clone().parse::<i64>().unwrap())
    } else {
        None
    }
}

#[derive(Serialize, Deserialize, Debug)]

pub struct TypeDef {
    pub account_address: String,
    pub module_name: String,
    pub struct_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct MoveOption<T> {
    pub vec: Vec<T>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]

pub struct MoveTable {
    pub inner: MoveTableHandle,
    pub length: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct MoveTableHandle {
    pub handle: String,
}



#[derive(Debug)]
pub enum OrganizationResource {
    CreateOrganization(OrganizationDto),
    CreateGovernance(GovernancesDto),
    CreateTreasury(TreasuryDto)
    
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemberDataType {
    pub member_metadata: MemberMetadata,
    pub role: String,
    pub status: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct MemberMetadata {
    pub proposal_created: i64,
    pub aptocracy_address: String
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrganizationWriteSet {
    MemberData(MemberDataType),
    ProposalData(ProposalDto),
    VoteOptionData(VoteOptionTableContent)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AcceptMembershipEvent {
    pub member_address: String,
    pub organization_address: String,
    pub member_status: i64,
    pub role: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VoteEvent {
    pub proposal_state: i32,
    pub voting_finalized_at: MoveOption<String>,
    pub proposal_id: String,
    pub proposal_content: ProposalMetadata,
    pub vote_options: Vec<String>,
    pub options_elected: Vec<bool>,
    pub member_address:String,
    pub vote_weight:String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RelinquishVoteEvent {
    pub member_address: String,
    pub vote_weight: String,
    pub vote_options: Vec<String>,
    pub proposal_id: String,
    pub proposal_content: ProposalMetadata,
    pub options_elected: Vec<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CancelProposalEvent {
    pub proposal_id: String,
    pub proposal_state: i32,
    pub proposal_content: ProposalMetadata,
    pub cancelled_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FinalizeVoteEvent {
    proposal_id: String,
    proposal_content: ProposalMetadata,
    proposal_state: i32,
    voting_finalized_at: String,
}


#[derive(Deserialize, Serialize, Debug)]
pub struct DepositRecordEvent {
    pub member_address: String,
    pub deposit_amount: String,
    pub accumulated_deposit_record_amount: String,
    pub treasury_metadata: TreasuryMetadata,
    pub treasury_address: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WithdrawEvent {
    pub member_address: String,
    pub withdraw_amount: String,
    pub accumulated_deposit_record_amount: String,
    pub treasury_metadata: TreasuryMetadata,
    pub treasury_address: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AptocracyEvent {
    CastVote(VoteEvent),
    RelinquishVote(RelinquishVoteEvent),
    CancelProposal(CancelProposalEvent),
    FinalizeVote(FinalizeVoteEvent),
    Deposit(DepositRecordEvent),
    Withdraw(WithdrawEvent),
    AcceptMembershipEvent(AcceptMembershipEvent)
}


impl OrganizationWriteSet {
    pub fn from_table_item_type(
        data_type: &str,
        data: &serde_json::Value,
    ) -> Result<Option<OrganizationWriteSet>> {
        match data_type {
            "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::organization::Member<0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::aptocracy::AptocracyMember>" => {
                serde_json::from_value(data.clone())
                .map(|inner| 
                    Some(OrganizationWriteSet::MemberData(inner))
                )
            },
             "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::proposals::Proposal<0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::aptocracy::AptocracyProposal>" => {
                serde_json::from_value(data.clone())
                .map(|inner| 
                    Some(OrganizationWriteSet::ProposalData(inner))
                )
            },
             "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::proposals::VoteOption" => {
                serde_json::from_value(data.clone())
                .map(|inner| 
                    Some(OrganizationWriteSet::VoteOptionData(inner))
                )
            },

            _ => Ok(None),
        }        
        .context(format!(
            "failed to parse type {}, data {:?} _test",
            data_type, data
        ))
    }
  
}



impl OrganizationResource {
    pub fn from_write_resource(
        write_resource: &WriteResource,
        txn_version: i64,
    ) -> Result<Option<Self>> {

        let type_str = format!(
            "{}::{}::{}",
            write_resource.data.typ.address,
            write_resource.data.typ.module,
            write_resource.data.typ.name
        );

        if !Self::is_resource_supported(type_str.as_str()) {
            return Ok(None);
        }

        let resource = MoveResource::from_write_resource(
            write_resource,
            0, // Placeholder, this isn't used anyway
            txn_version,
            0, // Placeholder, this isn't used anyway
        );
        Ok(Some(Self::from_resource(
            &type_str,
            resource.data.as_ref().unwrap(),
            txn_version,
        )?))
    }

    fn is_resource_supported(data_type: &str) -> bool {
        matches!(
            data_type,
            "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::treasury::Treasury" | "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::organization::Organization" | "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::organization::Governances"
        )
    }


    pub fn from_resource(
        data_type: &str,
        data: &serde_json::Value,
        txn_version: i64,
    ) -> Result<Self> {

        println!("data_type__processor {:?}", data_type);

        match data_type {
         "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::treasury::Treasury" => {
               serde_json::from_value(data.clone()).map(|inner| Some(OrganizationResource::CreateTreasury(inner)))
              },
        "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::organization::Organization" => {
                serde_json::from_value(data.clone())
                .map(|inner| Some(OrganizationResource::CreateOrganization(inner)))},
        "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::organization::Governances" => {
                serde_json::from_value(data.clone())
                .map(|inner| Some(OrganizationResource::CreateGovernance(inner)))},
            _ => Ok(None),
        }
        .context(format!(
            "version {} failed! failed to parse type {}, data {:?}",
            txn_version, data_type, data
        ))?
        .context(format!(
            "Resource unsupported! Call is_resource_supported first. version {} type {}",
            txn_version, data_type
        ))
    }

   
}


impl AptocracyEvent{
    pub fn from_event(data_type: &str, data: &serde_json::Value) -> Option<Self> {
        println!("DATA_TYPE_test {}",data_type);
        match data_type {
            "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::proposals::VoteEvent<0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::aptocracy::AptocracyProposal>" => {
                println!("DATA_test {}",data);
          
                if let Ok(vote_event)=serde_json::from_value(data.clone()){
                    println!("MATCHED_test {:?}",vote_event);
                    Some(AptocracyEvent::CastVote(vote_event))
                }else{
                    None
                }
            },
            "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::proposals::RelinquishVoteEvent<0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::aptocracy::AptocracyProposal>"=>{
                if let Ok(relinquish_vote)=serde_json::from_value(data.clone()){
                    println!("PARSED_data {:?}",relinquish_vote);
                    Some(AptocracyEvent::RelinquishVote(relinquish_vote))
                }else{
                    None
                }
            },
            "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::proposals::CancelProposalEvent<0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::aptocracy::AptocracyProposal>"=>{
                if let Ok(cancel_event)=serde_json::from_value(data.clone()){
                    println!("PARSED_test {:?}",cancel_event);
                    Some(AptocracyEvent::CancelProposal(cancel_event))
                }else{
                    None
                }
            },
            "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::proposals::FinalizeVoteEvent<0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::aptocracy::AptocracyProposal>"=>{
                if let Ok(finalized_vote)=serde_json::from_value(data.clone()){
                    println!("PARSED_test {:?}",finalized_vote);
                    Some(AptocracyEvent::FinalizeVote(finalized_vote))
                }else{
                    None
                }
            },
            "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::treasury::DepositEvent<0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::aptocracy::AptocracyTreasury>"=>{
                if let Ok(deposit)=serde_json::from_value(data.clone()){
                    Some(AptocracyEvent::Deposit(deposit))
                }else{
                    None
                }
            },
            "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::treasury::WithdrawEvent<0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::aptocracy::AptocracyTreasury>"=>{
                if let Ok(withdraw)=serde_json::from_value(data.clone()){
                    Some(AptocracyEvent::Withdraw(withdraw))
                }else{
                    None
                }
            },
             "0x0e38566f0ba66f0b913bd644b8044c7a77528d5ee0a1aede129709ab08b90bc1::organization::AcceptMembershipEvent" =>
             if let Ok(accept_membership) = serde_json::from_value(data.clone()) {
                Some(AptocracyEvent::AcceptMembershipEvent(accept_membership))
             } else {
                None
             }, 
            _ => None,
        }
    }
}


pub fn parse_move_string(value: &str) -> &str {
    &value[1..value.len() - 1]
}