use crate::models::aptocracy_models::aptocracy_utils::{
    AptocracyEvent, DepositRecordEvent, WithdrawEvent,
};
use crate::processors::aptocracy_processor::AptocracyProcessor;
use crate::{models::aptocracy_models::aptocracy_utils::OrganizationResource, schema::*};
use aptos_api_types::{Transaction as APITransaction, WriteSetChange as APIWriteSetChange};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct TreasuryDto {
    pub authority: String,
    pub signer_capability: MoveSignerCapability,
    pub treasury_index: i32,
    pub deposited_amount: String,
    pub treasury_metadata: TreasuryMetadata,
    pub treasury_coin: TreasuryCoin,
    pub treasury_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TreasuryMetadata {
    pub governance_id: String,
    pub aptocracy_address: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TreasuryCoin {
    pub account_address: String,
    pub module_name: String,
    pub struct_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoveSignerCapability {
    pub account: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoveSignerCapabilityInner {
    vec: Vec<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name=treasury)]
pub struct Treasury {
    pub treasury_address: String,
    pub aptocracy_address: String,
    pub authority: String,
    pub treasury_index: i32,
    pub deposited_amount: i64,
    pub treasury_coin: String,
    pub governance_id: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = deposit_record)]
pub struct DepositRecord {
    pub treasury_address: String,
    pub member_address: String,
    pub aptocracy_address: String,
    pub accumulated_amount: i64,
    pub last_deposit: NaiveDateTime,
}

impl Treasury {
    pub fn from_transaction(transaction: &APITransaction) -> (Vec<Treasury>, Vec<DepositRecord>) {
        let mut treasuries: Vec<Treasury> = vec![];
        let mut deposit_records: Vec<DepositRecord> = vec![];

        if let APITransaction::UserTransaction(user_txn) = &transaction {
            for wsc in user_txn.info.changes.iter() {
                if let APIWriteSetChange::WriteResource(treasury) = &wsc {
                    if let Some(OrganizationResource::CreateTreasury(treasury_dto)) =
                        OrganizationResource::from_write_resource(
                            treasury,
                            user_txn.info.version.0 as i64,
                        )
                        .unwrap()
                    {
                        treasuries.push(treasury_dto.from_dto());
                    }
                }
            }

            for event in user_txn.events.iter() {
                if let Some(event) =
                    AptocracyEvent::from_event(event.typ.to_string().as_str(), &event.data)
                {
                    if let AptocracyEvent::Deposit(deposit) = event {
                        deposit_records.push(deposit.from_dto());
                    } else {
                        if let AptocracyEvent::Withdraw(withdraw) = event {
                            deposit_records.push(withdraw.from_dto());
                        }
                    }
                }
            }
        }

        (treasuries, deposit_records)
    }

    pub fn get_treasury_resource() -> String {
        let module_address = AptocracyProcessor::get_module_address();

        let formated = format!("{}::treasury::Treasury", module_address);

        formated
    }
}

impl TreasuryDto {
    pub fn from_dto(&self) -> Treasury {
        Treasury {
            treasury_address: self.treasury_address.clone(),
            aptocracy_address: self.treasury_metadata.aptocracy_address.clone(),
            authority: self.authority.clone(),
            treasury_index: self.treasury_index,
            deposited_amount: self.deposited_amount.parse::<i64>().unwrap(),
            treasury_coin: serde_json::to_string::<TreasuryCoin>(&self.treasury_coin).unwrap(),
            governance_id: self.treasury_metadata.governance_id.parse::<i64>().unwrap(),
        }
    }
}

impl DepositRecordEvent {
    pub fn from_dto(self) -> DepositRecord {
        DepositRecord {
            treasury_address: self.treasury_address.clone(),
            member_address: self.member_address.clone(),
            aptocracy_address: self.treasury_metadata.aptocracy_address.clone(),
            accumulated_amount: self
                .accumulated_deposit_record_amount
                .parse::<i64>()
                .unwrap(),
            last_deposit: Utc::now().naive_utc(),
        }
    }
}

impl WithdrawEvent {
    pub fn from_dto(self) -> DepositRecord {
        DepositRecord {
            treasury_address: self.treasury_address.clone(),
            member_address: self.member_address.clone(),
            aptocracy_address: self.treasury_metadata.aptocracy_address.clone(),
            accumulated_amount: self
                .accumulated_deposit_record_amount
                .parse::<i64>()
                .unwrap(),
            last_deposit: Utc::now().naive_utc(),
        }
    }
}
