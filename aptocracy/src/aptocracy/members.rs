use aptos_indexer::schema::aptocracy_user::{self, dsl::*};
use aptos_indexer::schema::member::{self, dsl::*};
use diesel::prelude::*;
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject};
use serde::{Deserialize, Serialize};

use crate::helpers::map_to_f64;
use crate::{
    graphql::GraphQlContext,
    helpers::{parse_graphql_response, parse_socials, AptocracyParser},
};
#[derive(Insertable, Debug, Queryable, Clone, AsChangeset)]
#[diesel(table_name=aptocracy_user)]
pub struct AptocracyUser {
    pub user_address: String,
    pub email: Option<String>,
    pub socials: Option<String>,
    pub name: Option<String>,
}

impl AptocracyUser {
    pub fn save_user_data(
        conn: &mut PgConnection,
        user_data: AptocracyUserDto,
    ) -> FieldResult<AptocracyUserResponseDto> {
        let response = diesel::insert_into(aptocracy_user::table)
            .values(user_data.from())
            .get_result(conn);

        parse_graphql_response::<AptocracyUser, AptocracyUserResponseDto>(response)
    }

    pub fn update_user_data(
        conn: &mut PgConnection,
        user_data: AptocracyUserDto,
    ) -> FieldResult<AptocracyUserResponseDto> {
        let response = diesel::update(aptocracy_user::table)
            .filter(aptocracy_user::user_address.eq(user_data.user_address.clone()))
            .set(user_data.clone().from())
            .get_result(conn);

        parse_graphql_response::<AptocracyUser, AptocracyUserResponseDto>(response)
    }

    pub fn get_user_data(
        conn: &mut PgConnection,
        aptocracy_user_address: String,
    ) -> FieldResult<AptocracyUserResponseDto> {
        let response = aptocracy_user
            .filter(aptocracy_user::user_address.eq(aptocracy_user_address))
            .get_result(conn);

        parse_graphql_response::<AptocracyUser, AptocracyUserResponseDto>(response)
    }

    pub fn to_dto(self) -> AptocracyUserResponseDto {
        AptocracyUserResponseDto {
            user_address: self.user_address.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
            socials: parse_socials::<SocialNetworkResponse>(self),
        }
    }
}

#[derive(GraphQLInputObject, Clone)]
pub struct AptocracyUserDto {
    pub user_address: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub socials: Vec<SocialNetwork>,
}

#[derive(GraphQLEnum, Deserialize, Serialize, Clone)]
pub enum SocialType {
    Twitter,
    Discord,
    Telegram,
    Instagram,
    Tiktok,
}

#[derive(Deserialize, Serialize, GraphQLInputObject, Clone)]
pub struct SocialNetwork {
    pub social_type: SocialType,
    pub url: String,
}

impl AptocracyUserDto {
    fn from(self) -> AptocracyUser {
        AptocracyUser {
            user_address: self.user_address.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
            socials: self.parse_socials(),
        }
    }

    fn parse_socials(self) -> Option<String> {
        if self.socials.len() == 0 {
            None
        } else {
            Some(serde_json::to_string(&self.socials).unwrap())
        }
    }
}

impl AptocracyParser<AptocracyUserResponseDto> for AptocracyUser {
    fn from(self) -> AptocracyUserResponseDto {
        AptocracyUserResponseDto {
            user_address: self.user_address.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
            socials: parse_socials(self),
        }
    }
}

#[derive(Queryable, Clone, Debug)]
pub struct AptocracyMember {
    pub member_address: String,
    pub aptocracy_address: String,
    pub role: String,
    pub status: Option<i64>,
    pub proposal_created: Option<i64>,
}

impl AptocracyMember {
    // pub fn get_all_members_for_aptocracy(
    //     conn: &mut PgConnection,
    //     wanted_aptocracy_address: String,
    // ) -> FieldResult<Vec<AptocracyMemberDto>> {
    //     let response = member
    //         .filter(member::aptocracy_address.eq(wanted_aptocracy_address))
    //         .left_join(aptocracy_user.on(member::member_address.eq(aptocracy_user::user_address)))
    //         .load::<(AptocracyMember, Option<AptocracyUser>)>(conn);

    //     let parsed_response = match response {
    //         Ok(data) => FieldResult::Ok(Self::parse_members_response(data)),
    //         Err(e) => FieldResult::Err(FieldError::from(e)),
    //     };

    //     parsed_response
    // }

    // pub fn get_member_for_aptocracy(
    //     conn: &mut PgConnection,
    //     wanted_aptocracy_address: String,
    //     wanted_member_address: String,
    // ) -> FieldResult<AptocracyMemberDto> {
    //     let response = member
    //         .filter(member::aptocracy_address.eq(wanted_aptocracy_address))
    //         .filter(member::member_address.eq(wanted_member_address))
    //         .left_join(aptocracy_user.on(member::member_address.eq(aptocracy_user::user_address)))
    //         .get_result::<(AptocracyMember, Option<AptocracyUser>)>(conn);

    //     let parsed_response = match response {
    //         Ok(data) => FieldResult::Ok(Self::parse_member_response(data)),
    //         Err(e) => FieldResult::Err(FieldError::from(e)),
    //     };
    //     parsed_response
    // }

    fn parse_members_response(
        data: Vec<(AptocracyMember, Option<AptocracyUser>)>,
    ) -> Vec<AptocracyMemberDto> {
        let mut members: Vec<AptocracyMemberDto> = vec![];

        for (member_data, user_data) in data.iter() {
            let user_data = if let Some(user_detail_data) = &user_data {
                Some(user_detail_data.clone().to_dto())
            } else {
                None
            };

            members.push(AptocracyMemberDto {
                aptocracy_address: member_data.aptocracy_address.clone(),
                member_address: member_data.member_address.clone(),
                member_data: user_data,
                status: map_to_f64(member_data.status),
                proposal_created: map_to_f64(member_data.proposal_created),
                role: member_data.role.clone(),
            })
        }

        members
    }

    fn parse_member_response(data: (AptocracyMember, Option<AptocracyUser>)) -> AptocracyMemberDto {
        let (member_data, user_data) = data;
        let user_data = if let Some(user_detail_data) = &user_data {
            Some(user_detail_data.clone().to_dto())
        } else {
            None
        };

        AptocracyMemberDto {
            aptocracy_address: member_data.aptocracy_address.clone(),
            member_address: member_data.member_address.clone(),
            member_data: user_data,
            status: map_to_f64(member_data.status),
            proposal_created: map_to_f64(member_data.proposal_created),
            role: member_data.role.clone(),
        }
    }
}

#[derive(GraphQLObject)]
pub struct AptocracyMemberDto {
    pub member_address: String,
    pub aptocracy_address: String,
    pub role: String,
    pub status: Option<f64>,
    pub proposal_created: Option<f64>,
    pub member_data: Option<AptocracyUserResponseDto>,
}

#[derive(GraphQLObject)]
pub struct AptocracyUserResponseDto {
    pub user_address: String,
    pub socials: Vec<SocialNetworkResponse>,
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, GraphQLObject)]
pub struct SocialNetworkResponse {
    pub social_type: SocialType,
    pub url: String,
}
