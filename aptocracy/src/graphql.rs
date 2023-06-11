use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode};

use crate::{
    aptocracy::{
        aptocracy::{Aptocracy, AptocracyDto, UpdateAptocracy},
        governance::{Governance, GovernanceDto},
        members::{
            AptocracyMember, AptocracyMemberDto, AptocracyUser, AptocracyUserDto,
            AptocracyUserResponseDto,
        },
    },
    db::PgPool,
    proposals::proposal::{Proposal, ProposalDto},
    treasury::treasury::{DepoitRecordDto, DepositRecord, Treasury, TreasuryDto},
};

pub struct GraphQlContext {
    pub pool: PgPool,
}

impl juniper::Context for GraphQlContext {}

pub struct Query;

#[graphql_object(Context=GraphQlContext)]
impl Query {
    #[graphql(
        name = "getAllAptocracies",
        description = "Fetching all aptocracies created on platform."
    )]
    pub fn get_all_aptocracies(ctx: &GraphQlContext) -> FieldResult<Vec<AptocracyDto>> {
        let conn = &mut ctx.pool.get().unwrap();
        Aptocracy::get_all_aptocracies(conn)
    }

    #[graphql(
        name = "getAptocracy",
        description = "Fetching aptocracy data by address"
    )]
    pub fn get_aptocracy_by_address(
        ctx: &GraphQlContext,
        address: String,
    ) -> FieldResult<AptocracyDto> {
        Aptocracy::get_by_address(&mut ctx.pool.get().unwrap(), address)
    }

    #[graphql(
        name = "checkIfNameIsTaken",
        description = "Cheks whether name of aptocracy already exists."
    )]
    pub fn check_if_name_exists(ctx: &GraphQlContext, name: String) -> FieldResult<bool> {
        Aptocracy::check_if_name_exists(&mut ctx.pool.get().unwrap(), name);
        FieldResult::Ok(true)
    }

    #[graphql(
        name = "getGovernances",
        description = "Fetches all governances for aptocracy"
    )]
    pub fn get_governances(
        ctx: &GraphQlContext,
        aptocracy_address: String,
    ) -> FieldResult<Vec<GovernanceDto>> {
        Governance::get_all_aptocracy_governances(&mut ctx.pool.get().unwrap(), aptocracy_address)
    }

    #[graphql(
        name = "getGovernanceById",
        description = "Fetches governance by aptocracy address and id"
    )]
    pub fn get_governance_by_id(
        ctx: &GraphQlContext,
        aptocracy_address: String,
        governance_id: i32,
    ) -> FieldResult<GovernanceDto> {
        Governance::get_governance_by_id(
            &mut ctx.pool.get().unwrap(),
            aptocracy_address,
            governance_id,
        )
    }

    // #[graphql(
    //     name = "getAllAptocracyMembers",
    //     description = "Fetches all members of one aptocracy"
    // )]
    // pub fn get_all_members_for_aptocracy(
    //     ctx: &GraphQlContext,
    //     aptocracy_address: String,
    // ) -> FieldResult<Vec<AptocracyMemberDto>> {
    //     AptocracyMember::get_all_members_for_aptocracy(
    //         &mut ctx.pool.get().unwrap(),
    //         aptocracy_address,
    //     )
    // }

    // #[graphql(
    //     name = "getAptocracyMemberByAddress",
    //     description = "Fetch member for aptocracy"
    // )]
    // pub fn get_member_for_aptocracy(
    //     ctx: &GraphQlContext,
    //     aptocracy_address: String,
    //     member_address: String,
    // ) -> FieldResult<AptocracyMemberDto> {
    //     AptocracyMember::get_member_for_aptocracy(
    //         &mut ctx.pool.get().unwrap(),
    //         aptocracy_address,
    //         member_address,
    //     )
    // }

    #[graphql(name = "getUserData", description = "Fetches basic data about user")]
    pub fn get_user_data(
        ctx: &GraphQlContext,
        user_address: String,
    ) -> FieldResult<AptocracyUserResponseDto> {
        AptocracyUser::get_user_data(&mut ctx.pool.get().unwrap(), user_address)
    }

    #[graphql(name = "getAllTreasuryProposals")]
    pub fn get_all_aptocracy_proposals(
        ctx: &GraphQlContext,
        treasury_address: String,
    ) -> FieldResult<Vec<ProposalDto>> {
        Proposal::get_proposals_for_treasury(&mut ctx.pool.get().unwrap(), treasury_address)
    }

    #[graphql(
        name = "getSingleProposalForTreasury",
        description = "Fetches single proposal"
    )]
    pub fn get_proposal(
        ctx: &GraphQlContext,
        treasury_address: String,
        proposal_id: f64,
    ) -> FieldResult<ProposalDto> {
        Proposal::get_signle_proposal_for_treasury(
            &mut ctx.pool.get().unwrap(),
            treasury_address,
            proposal_id as i64,
        )
    }

    #[graphql(
        name = "getTreasuriesForAptocracy",
        description = "Fetches all treasuries for given aptocracy"
    )]
    pub fn get_treasuries(
        ctx: &GraphQlContext,
        aptocracy_address: String,
    ) -> FieldResult<Vec<TreasuryDto>> {
        Treasury::get_treasuies_for_aptocracy(&mut ctx.pool.get().unwrap(), aptocracy_address)
    }

    #[graphql(
        name = "getTreasuryByIndex",
        description = "Fetches all treasuries for given aptocracy"
    )]
    pub fn get_treasury_by_index(
        ctx: &GraphQlContext,
        aptocracy_address: String,
        index: i32,
    ) -> FieldResult<TreasuryDto> {
        Treasury::get_by_index(&mut ctx.pool.get().unwrap(), index, aptocracy_address)
    }

    #[graphql(
        name = "getDepositRecords",
        description = "Fetches all deposit records per treasury"
    )]
    pub fn get_deposit_records(
        ctx: &GraphQlContext,
        treasury_address: String,
    ) -> FieldResult<Vec<DepoitRecordDto>> {
        DepositRecord::get_all_deposit_records_for_treasury(
            &mut ctx.pool.get().unwrap(),
            treasury_address,
        )
    }
    #[graphql(
        name = "getAllProposalsForAptocracy",
        description = "Fetches all proposals for aptocracy"
    )]
    pub fn get_all_aptocracy_proposals(
        ctx: &GraphQlContext,
        aptocracy_address: String,
    ) -> FieldResult<Vec<ProposalDto>> {
        Proposal::get_proposals_for_aptocracy(&mut ctx.pool.get().unwrap(), aptocracy_address)
    }

    #[graphql(
        name = "getSingleProposalForAptocracy",
        description = "Fetches proposal by its id for given aptocracy"
    )]
    pub fn get_signle_proposal_for_aptocracy(
        ctx: &GraphQlContext,
        aptocracy_address: String,
        proposal_id: f64,
    ) -> FieldResult<ProposalDto> {
        Proposal::get_signle_proposal_for_aptocracy(
            &mut ctx.pool.get().unwrap(),
            aptocracy_address,
            proposal_id as i64,
        )
    }

    #[graphql(
        name = "getLatestAptocracyProposal",
        description = "Fetches most recent proposal in aptocracy"
    )]
    pub fn get_latest_proposal_in_aptocracy(
        ctx: &GraphQlContext,
        aptocracy_address: String,
    ) -> FieldResult<ProposalDto> {
        Proposal::get_latest_proposal_in_aptocracy(&mut ctx.pool.get().unwrap(), aptocracy_address)
    }
}

pub struct Mutation;
#[graphql_object(Context=GraphQlContext)]
impl Mutation {
    #[graphql(
        name = "saveAptocracyUser",
        description = "Stores data about aptocracy user"
    )]
    pub fn save_aptocracy_user(
        ctx: &GraphQlContext,
        user_data: AptocracyUserDto,
    ) -> FieldResult<AptocracyUserResponseDto> {
        AptocracyUser::save_user_data(&mut ctx.pool.get().unwrap(), user_data)
    }

    #[graphql(
        name = "updateUserData",
        description = "Updates already existing user in database"
    )]
    pub fn update_user_data(
        ctx: &GraphQlContext,
        user_dto: AptocracyUserDto,
    ) -> FieldResult<AptocracyUserResponseDto> {
        AptocracyUser::update_user_data(&mut ctx.pool.get().unwrap(), user_dto)
    }

    #[graphql(
        name = "updateAptocracyData",
        description = "Updates image and description for given aptocracy"
    )]
    pub async fn update_aptocracy_data(
        ctx: &GraphQlContext,
        aptocracy_data: UpdateAptocracy,
    ) -> FieldResult<AptocracyDto> {
        Aptocracy::update_aptocracy_data(&mut ctx.pool.get().unwrap(), aptocracy_data).await
    }
}

pub type AptocracySchema = RootNode<'static, Query, Mutation, EmptySubscription<GraphQlContext>>;

pub fn create_schema() -> AptocracySchema {
    AptocracySchema::new(Query, Mutation, EmptySubscription::new())
}
