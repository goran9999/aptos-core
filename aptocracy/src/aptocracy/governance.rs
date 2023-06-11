use actix_rt::signal::unix;
use aptos_indexer::schema::governance::{self, dsl::*};
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection;
use juniper::{FieldResult, GraphQLObject};

use crate::helpers::parse_graphql_response;
use crate::helpers::parse_unix_option;
use crate::helpers::AptocracyParser;
#[derive(Queryable, Clone)]
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

impl Governance {
    pub fn get_all_aptocracy_governances(
        conn: &mut PgConnection,
        wanted_aptocracy_address: String,
    ) -> FieldResult<Vec<GovernanceDto>> {
        let data = governance
            .filter(governance::aptocracy_address.eq(wanted_aptocracy_address))
            .load::<Governance>(conn);

        parse_graphql_response::<Vec<Governance>, Vec<GovernanceDto>>(data)
    }

    pub fn get_governance_by_id(
        conn: &mut PgConnection,
        wanted_aptocracy_address: String,
        wanted_governance_id: i32,
    ) -> FieldResult<GovernanceDto> {
        let data = governance
            .filter(governance::aptocracy_address.eq(wanted_aptocracy_address))
            .filter(governance::governance_id.eq(wanted_governance_id as i64))
            .get_result(conn);

        parse_graphql_response::<Governance, GovernanceDto>(data)
    }
}

#[derive(GraphQLObject)]
pub struct GovernanceDto {
    pub aptocracy_address: String,
    pub governance_id: f64,
    pub max_voting_time: f64,
    pub quorum: f64,
    pub approval_quorum: f64,
    pub early_tipping: bool,
    pub valid_from: NaiveDateTime,
    pub valid_to: Option<NaiveDateTime>,
}

impl AptocracyParser<GovernanceDto> for Governance {
    fn from(self) -> GovernanceDto {
        GovernanceDto {
            aptocracy_address: self.aptocracy_address,
            governance_id: self.governance_id as f64,
            max_voting_time: self.max_voting_time as f64,
            quorum: self.quorum as f64,
            approval_quorum: self.approval_quorum as f64,
            early_tipping: self.early_tipping,
            valid_from: NaiveDateTime::from_timestamp_opt(self.valid_from, 0).unwrap(),
            valid_to: parse_unix_option(self.valid_to),
        }
    }
}

impl AptocracyParser<Vec<GovernanceDto>> for Vec<Governance> {
    fn from(self) -> Vec<GovernanceDto> {
        self.iter().map(|gov| gov.clone().from()).collect()
    }
}
