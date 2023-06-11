use crate::{
    models::aptocracy_models::aptocracy_utils::{AptocracyEvent, OrganizationWriteSet},
    schema::member,
    util::parse_timestamp,
};
use aptos_api_types::{
    Transaction as APITransaction, WriteSetChange as APIWriteSetChange,
    WriteTableItem as ApiWriteTableItem,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, AsChangeset, Deserialize, Serialize, Insertable)]
#[diesel(table_name = member)]
pub struct Member {
    pub member_address: String,
    pub aptocracy_address: String,
    pub role: String,
    pub status: i64,
    pub proposal_created: i64,
}

impl Member {
    pub fn from_transaction(transaction: &APITransaction) -> Vec<Member> {
        let mut member_infos: Vec<Member> = vec![];
        if let APITransaction::UserTransaction(user_txn) = transaction {
            let txn_version = user_txn.info.version.0 as i64;

            for wsc in user_txn.info.changes.iter() {
                if let APIWriteSetChange::WriteTableItem(write_table_item) = wsc {
                    if let Some(member_info) =
                        Self::from_write_table_item(&write_table_item).unwrap()
                    {
                        member_infos.push(member_info)
                    }
                }
            }

            for event in user_txn.events.iter() {
                let event_type = event.typ.to_string();

                if let Some(AptocracyEvent::AcceptMembershipEvent(inner)) =
                    AptocracyEvent::from_event(event_type.as_str(), &event.data)
                {
                    member_infos.push(Self {
                        aptocracy_address: inner.organization_address,
                        role: inner.role,
                        status: inner.member_status,
                        proposal_created: 0,
                        member_address: inner.member_address,
                    });
                }
            }
        }

        member_infos
    }

    pub fn from_write_table_item(table_item: &ApiWriteTableItem) -> anyhow::Result<Option<Self>> {
        let table_item_data = table_item.data.as_ref().unwrap();

        if let Some(OrganizationWriteSet::MemberData(inner)) =
            OrganizationWriteSet::from_table_item_type(
                table_item_data.value_type.as_str(),
                &table_item_data.value,
            )
            .unwrap()
        {
            return Ok(Some(Member {
                aptocracy_address: inner.member_metadata.aptocracy_address,
                member_address: table_item.key.to_string(),
                role: inner.role,
                proposal_created: inner.member_metadata.proposal_created,
                status: inner.status,
            }));
        }
        Ok(None)
    }
}
