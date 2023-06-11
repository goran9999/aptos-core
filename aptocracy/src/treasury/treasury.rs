use aptos_indexer::schema::{
    deposit_record::{self, dsl::*},
    treasury::{self, dsl::*},
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use juniper::{FieldResult, GraphQLObject};

use crate::helpers::{parse_graphql_response, AptocracyParser};
#[derive(Queryable, Clone)]
pub struct Treasury {
    pub treasury_address: String,
    pub aptocracy_address: String,
    pub authority: String,
    pub treasury_index: i32,
    pub deposited_amount: i64,
    pub treasury_coin: String,
    pub governance_id: i64,
}

#[derive(GraphQLObject, Clone)]
pub struct TreasuryDto {
    pub treasury_address: String,
    pub aptocracy_address: String,
    pub authority: String,
    pub treasury_index: i32,
    pub deposited_amount: f64,
    pub treasury_coin: String,
    pub governance_id: f64,
}

#[derive(Debug, Queryable, Clone)]

pub struct DepositRecord {
    pub treasury_address: String,
    pub member_address: String,
    pub aptocracy_address: String,
    pub accumulated_amount: i64,
    pub last_deposit: NaiveDateTime,
}

#[derive(GraphQLObject, Clone)]
pub struct DepoitRecordDto {
    pub treasury_address: String,
    pub member_address: String,
    pub aptocracy_address: String,
    pub accumulated_amount: f64,
    pub last_deposit: NaiveDateTime,
}

impl Treasury {
    pub fn get_treasuies_for_aptocracy(
        conn: &mut PgConnection,
        wanted_aptocracy: String,
    ) -> FieldResult<Vec<TreasuryDto>> {
        let response = treasury
            .filter(treasury::aptocracy_address.eq(wanted_aptocracy))
            .load::<Treasury>(conn);

        parse_graphql_response::<Vec<Treasury>, Vec<TreasuryDto>>(response)
    }

    pub fn get_by_index(
        conn: &mut PgConnection,
        index: i32,
        wanted_aptocracy: String,
    ) -> FieldResult<TreasuryDto> {
        let response = treasury
            .filter(treasury::aptocracy_address.eq(wanted_aptocracy))
            .filter(treasury::treasury_index.eq(index))
            .get_result(conn);

        parse_graphql_response::<Treasury, TreasuryDto>(response)
    }
}

impl AptocracyParser<TreasuryDto> for Treasury {
    fn from(self) -> TreasuryDto {
        TreasuryDto {
            treasury_address: self.treasury_address,
            aptocracy_address: self.aptocracy_address,
            authority: self.authority,
            treasury_index: self.treasury_index,
            deposited_amount: self.deposited_amount as f64,
            treasury_coin: self.treasury_coin,
            governance_id: self.governance_id as f64,
        }
    }
}

impl DepositRecord {
    pub fn get_all_deposit_records_for_treasury(
        conn: &mut PgConnection,
        wanted_treasury_address: String,
    ) -> FieldResult<Vec<DepoitRecordDto>> {
        let response = deposit_record
            .filter(deposit_record::treasury_address.eq(wanted_treasury_address))
            .load::<DepositRecord>(conn);

        parse_graphql_response::<Vec<DepositRecord>, Vec<DepoitRecordDto>>(response)
    }
}

impl AptocracyParser<Vec<TreasuryDto>> for Vec<Treasury> {
    fn from(self) -> Vec<TreasuryDto> {
        self.iter().map(|item| item.clone().from()).collect()
    }
}

impl AptocracyParser<DepoitRecordDto> for DepositRecord {
    fn from(self) -> DepoitRecordDto {
        DepoitRecordDto {
            treasury_address: self.treasury_address,
            member_address: self.member_address,
            aptocracy_address: self.aptocracy_address,
            accumulated_amount: self.accumulated_amount as f64,
            last_deposit: self.last_deposit,
        }
    }
}

impl AptocracyParser<Vec<DepoitRecordDto>> for Vec<DepositRecord> {
    fn from(self) -> Vec<DepoitRecordDto> {
        self.iter().map(|item| item.clone().from()).collect()
    }
}
