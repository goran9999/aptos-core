use crate::database::{execute_with_better_error, PgPoolConnection};
use crate::models::aptocracy_models::aptocracy_utils::AptocracyEvent;
use crate::models::aptocracy_models::organization::members::Member;
use crate::models::aptocracy_models::organization::organization::Governance;
use crate::models::aptocracy_models::proposal::proposals::Proposal;
use crate::models::aptocracy_models::proposal::vote_options::{
    ExecutionStep, VoteOption, VoteOptionTableItemDto, VoteRecord,
};

use crate::models::aptocracy_models::treasury::treasury::{DepositRecord, Treasury};
use crate::schema;
use crate::schema::governance::aptocracy_address as governance_aptocracy_addr;
use crate::schema::governance::governance_id;
use crate::schema::member::aptocracy_address;
use crate::schema::member::member_address;

use crate::{
    database::PgDbPool,
    indexer::{
        errors::TransactionProcessingError, processing_result::ProcessingResult,
        transaction_processor::TransactionProcessor,
    },
    models::aptocracy_models::organization::organization::Organization,
};
use aptos_api_types::Transaction as APITransaction;
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{PgConnection, RunQueryDsl};
use std::fmt::Debug;
pub const NAME: &str = "aptocracy_processor";

pub struct AptocracyProcessor {
    pub connection_pool: PgDbPool,
}

impl Debug for AptocracyProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "CoinTransactionProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}

impl AptocracyProcessor {
    pub fn new(connection_pool: PgDbPool) -> Self {
        Self { connection_pool }
    }

    pub fn get_module_address() -> String {
        dotenv::dotenv().ok();

        let module_address = std::env::var("MODULE_ADDRESS").expect("Failed to load env variable");

        println!("MODULE ADDR {}", module_address);

        module_address.to_string()
    }
}

#[async_trait]
impl TransactionProcessor for AptocracyProcessor {
    fn name(&self) -> &'static str {
        NAME
    }

    fn connection_pool(&self) -> &PgDbPool {
        &self.connection_pool
    }

    async fn process_transactions(
        &self,
        transactions: Vec<APITransaction>,
        start_version: u64,
        end_version: u64,
    ) -> Result<ProcessingResult, TransactionProcessingError> {
        let mut organizations: Vec<Organization> = vec![];
        let mut members: Vec<Member> = vec![];
        let mut governances: Vec<Governance> = vec![];
        let mut conn = self.get_conn();
        let mut proposals: Vec<Proposal> = vec![];
        let mut proposal_options_data: Vec<VoteOptionTableItemDto> = vec![];
        let mut proposal_events_data: Vec<AptocracyEvent> = vec![];
        let mut treasuries: Vec<Treasury> = vec![];
        let mut deposit_records: Vec<DepositRecord> = vec![];

        for tx in transactions.iter() {
            let (mut org, mut gov) = Organization::from_transaction(tx);
            let mut mem = Member::from_transaction(tx);
            let (mut proposal, mut proposal_options, mut proposal_events) =
                Proposal::from_transaction(tx);

            let (mut treasuries_data, mut deposit_records_data) = Treasury::from_transaction(tx);
            proposals.append(&mut proposal);
            proposal_options_data.append(&mut proposal_options);
            members.append(&mut mem);
            organizations.append(&mut org);
            governances.append(&mut gov);
            proposal_events_data.append(&mut proposal_events);
            treasuries.append(&mut treasuries_data);
            deposit_records.append(&mut deposit_records_data);
        }

        let tx_result = insert_to_db(
            &mut conn,
            organizations,
            members,
            governances,
            proposals,
            proposal_options_data,
            proposal_events_data,
            treasuries,
            deposit_records,
        );

        match tx_result {
            Ok(_) => Ok(ProcessingResult::new(
                self.name(),
                start_version,
                end_version,
            )),
            Err(err) => Err(TransactionProcessingError::TransactionCommitError((
                anyhow::Error::from(err),
                start_version,
                end_version,
                self.name(),
            ))),
        }
    }
}

fn insert_ogranizations(
    conn: &mut PgConnection,
    items_to_insert: &[Organization],
) -> Result<(), diesel::result::Error> {
    for org in items_to_insert.iter() {
        execute_with_better_error(
            conn,
            diesel::insert_into(schema::organization::table)
                .values(org)
                .on_conflict(schema::organization::address)
                .do_update()
                .set(org),
            None,
        )?;
    }
    Ok(())
}

fn insert_members(
    conn: &mut PgConnection,
    items_to_insert: &[Member],
) -> Result<(), diesel::result::Error> {
    for member in items_to_insert.iter() {
        execute_with_better_error(
            conn,
            diesel::insert_into(schema::member::table)
                .values(member)
                .on_conflict((member_address, aptocracy_address))
                .do_update()
                .set(member),
            None,
        )?;
    }
    Ok(())
}

fn insert_governances(
    conn: &mut PgConnection,
    items_to_insert: &[Governance],
) -> Result<(), diesel::result::Error> {
    for governance in items_to_insert.iter() {
        execute_with_better_error(
            conn,
            diesel::insert_into(schema::governance::table)
                .values(governance)
                .on_conflict((governance_id, governance_aptocracy_addr))
                .do_update()
                .set(governance),
            None,
        )?;
    }
    Ok(())
}

fn insert_proposal_data(
    conn: &mut PgConnection,
    proposals: Vec<Proposal>,
    proposal_options: Vec<VoteOptionTableItemDto>,
) -> Result<(), diesel::result::Error> {
    for (_index, proposal_data) in proposals.iter().enumerate() {
        let proposal_id = diesel::insert_into(schema::proposal::table)
            .values(proposal_data)
            .on_conflict((
                schema::proposal::proposal_id,
                schema::proposal::treasury_address,
            ))
            .do_update()
            .set((
                schema::proposal::state.eq(proposal_data.state),
                schema::proposal::cancelled_at.eq(proposal_data.cancelled_at),
                schema::proposal::voting_finalized_at.eq(proposal_data.voting_finalized_at),
                schema::proposal::executed_at.eq(proposal_data.executed_at),
            ))
            .returning(schema::proposal::id)
            .get_result(conn)
            .unwrap();
        for vote_option_data in proposal_options.iter() {
            let vote_option = VoteOption::from(
                vote_option_data,
                proposal_data.treasury_address.clone(),
                proposal_id,
            );
            let vote_options_id = diesel::insert_into(schema::vote_options::table)
                .values(vote_option.clone())
                .on_conflict((
                    schema::vote_options::proposal_id,
                    schema::vote_options::option,
                ))
                .do_update()
                .set((
                    schema::vote_options::vote_weight.eq(vote_option.vote_weight),
                    schema::vote_options::option_elected.eq(vote_option.option_elected.clone()),
                ))
                .returning(schema::vote_options::id)
                .get_result(conn)
                .unwrap();
            println!("VO_ID _test__processor {:?}", vote_options_id);

            let execution_steps = vote_option_data
                .value
                .execution_steps
                .iter()
                .map(|step| {
                    ExecutionStep::from(
                        step,
                        vote_options_id,
                        proposal_data.proposal_id,
                        proposal_data.treasury_address.clone(),
                    )
                })
                .collect::<Vec<ExecutionStep>>();
            //TODO:may be issue beceause we don't call .execute(conn)

            println!("TEST_test {:?}", execution_steps);

            //ADD UNIQUE CONSTRAINT
            for execution_step in execution_steps.iter() {
                execute_with_better_error(
                    conn,
                    diesel::insert_into(schema::execution_step::table).values(execution_step),
                    // .on_conflict((
                    //     schema::execution_step::vote_option_id,
                    //     schema::execution_step::execution_hash,
                    // ))
                    // .do_update()
                    // .set(execution_step),
                    None,
                )
                .unwrap();
            }
        }
    }

    Ok(())
}

fn insert_proposal_events(conn: &mut PgConnection, events: Vec<AptocracyEvent>) {
    for event in events.iter() {
        insert_vote_record(conn, event);
    }
}

fn insert_vote_record(conn: &mut PgConnection, vote_event: &AptocracyEvent) {
    let response = match vote_event {
        AptocracyEvent::CastVote(event) => execute_with_better_error(
            conn,
            diesel::insert_into(schema::vote_record::table).values(VoteRecord {
                member_address: event.member_address.clone(),
                proposal_id: event.proposal_id.parse::<i64>().unwrap(),
                treasury_address: event.proposal_content.treasury_address.clone(),
                voter_weight: event.vote_weight.parse::<i64>().unwrap(),
                voted_at: Utc::now().naive_utc(),
                elected_options: event.vote_options.clone(),
            }),
            None,
        ),
        AptocracyEvent::RelinquishVote(event) => execute_with_better_error(
            conn,
            diesel::delete(schema::vote_record::table)
                .filter(schema::vote_record::member_address.eq(event.member_address.clone()))
                .filter(
                    schema::vote_record::proposal_id.eq(event.proposal_id.parse::<i64>().unwrap()),
                )
                .filter(
                    schema::vote_record::treasury_address
                        .eq(event.proposal_content.treasury_address.clone()),
                ),
            None,
        ),
        _ => Ok(0_usize),
    };
    println!("RESPONSE_test {:?}", response);
}

fn insert_treasury(
    conn: &mut PgConnection,
    items_to_insert: &[Treasury],
) -> Result<(), diesel::result::Error> {
    for treasury in items_to_insert.iter() {
        execute_with_better_error(
            conn,
            diesel::insert_into(schema::treasury::table)
                .values(treasury)
                .on_conflict(schema::treasury::treasury_address)
                .do_update()
                .set(schema::treasury::deposited_amount.eq(treasury.deposited_amount)),
            None,
        )?;
    }

    Ok(())
}

fn insert_deposit_records(
    conn: &mut PgConnection,
    deposit_records: Vec<DepositRecord>,
) -> Result<(), diesel::result::Error> {
    for deposit_record in deposit_records.iter() {
        let response = execute_with_better_error(
            conn,
            diesel::insert_into(schema::deposit_record::table)
                .values(deposit_record)
                .on_conflict((
                    schema::deposit_record::member_address,
                    schema::deposit_record::treasury_address,
                ))
                .do_update()
                .set(
                    schema::deposit_record::accumulated_amount
                        .eq(deposit_record.accumulated_amount),
                ),
            None,
        );
        println!("GOT_RESPONSE_test {:?}", response);
    }

    Ok(())
}

fn insert_to_db_impl(
    conn: &mut PgConnection,
    organizations: &[Organization],
    members: &[Member],
    governances: &[Governance],
    proposals: Vec<Proposal>,
    proposal_options: Vec<VoteOptionTableItemDto>,
    proposal_events: Vec<AptocracyEvent>,
    treasuries: &[Treasury],
    deposit_records: Vec<DepositRecord>,
) -> Result<(), diesel::result::Error> {
    insert_ogranizations(conn, organizations)?;
    insert_proposal_data(conn, proposals, proposal_options);
    insert_proposal_events(conn, proposal_events);
    insert_treasury(conn, treasuries);
    insert_deposit_records(conn, deposit_records);
    insert_members(conn, members);
    insert_governances(conn, governances);
    Ok(())
}

fn insert_to_db(
    conn: &mut PgPoolConnection,
    organizations: Vec<Organization>,
    members: Vec<Member>,
    governances: Vec<Governance>,
    proposal: Vec<Proposal>,
    proposal_options: Vec<VoteOptionTableItemDto>,
    proposal_events: Vec<AptocracyEvent>,
    treasuries: Vec<Treasury>,
    deposit_records: Vec<DepositRecord>,
) -> Result<(), diesel::result::Error> {
    match conn
        .build_transaction()
        .read_write()
        .run::<_, Error, _>(|pg_conn| insert_to_db_impl(pg_conn, &organizations,&members, &governances, proposal,proposal_options,proposal_events,&treasuries,deposit_records))
    {
        Ok(_) => Ok(()),
        Err(_) => Ok(())
        // conn
        //     .build_transaction()
        //     .read_write()
        //     .run::<_, Error, _>(|pg_conn| {
        //         let coin_activities = clean_data_for_db(organizations, true);
        //         insert_to_db_impl(pg_conn, &organizations)
        //     }),
    }
}
