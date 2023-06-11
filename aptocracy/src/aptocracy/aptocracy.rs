use aptos_indexer::schema::organization::{self, dsl::*};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::Queryable;
use juniper::{FieldResult, GraphQLInputObject, GraphQLObject};

use crate::helpers::map_to_f64;
use crate::helpers::map_to_governing_collection_info;
use crate::helpers::map_to_role_config;
use crate::helpers::parse_graphql_response;
use crate::helpers::upload_image_aws;
use crate::helpers::AptocracyParser;
#[derive(Queryable, Clone)]
pub struct Aptocracy {
    pub address: String,
    pub name: String,
    pub creator: String,
    pub default_role: String,
    pub governing_coin: String,
    pub governing_collection_info: String,
    pub invite_only: bool,
    pub main_governance: Option<i64>,
    pub max_voter_weight: Option<i64>,
    pub org_type: String,
    pub treasury_count: i32,
    pub role_config: String,
    pub created_at: NaiveDateTime,
    pub image: Option<String>,
    pub description: Option<String>,
    pub main_treasury: Option<String>,
}

impl Aptocracy {
    pub fn get_all_aptocracies(conn: &mut PgConnection) -> FieldResult<Vec<AptocracyDto>> {
        let data = organization.load::<Aptocracy>(conn);

        parse_graphql_response::<Vec<Aptocracy>, Vec<AptocracyDto>>(data)
    }

    pub fn get_by_address(
        conn: &mut PgConnection,
        aptocracy_address: String,
    ) -> FieldResult<AptocracyDto> {
        let data = organization
            .filter(organization::address.eq(aptocracy_address))
            .get_result::<Aptocracy>(conn);

        parse_graphql_response::<Aptocracy, AptocracyDto>(data)
    }

    pub fn check_if_name_exists(conn: &mut PgConnection, org_name: String) -> FieldResult<bool> {
        let data = organization
            .filter(organization::name.eq(org_name))
            .get_result::<Aptocracy>(conn);

        if let Ok(organization_data) = data {
            FieldResult::Ok(true)
        } else {
            FieldResult::Ok(false)
        }
    }

    pub async fn update_aptocracy_data(
        conn: &mut PgConnection,
        aptocracy_data: UpdateAptocracy,
    ) -> FieldResult<AptocracyDto> {
        let response;
        if !aptocracy_data.image_base_64.is_empty() {
            let uploaded_image = upload_image_aws(aptocracy_data.image_base_64).await;
            response = diesel::update(organization::table)
                .filter(organization::address.eq(aptocracy_data.aptocracy_address))
                .set((
                    organization::image.eq(uploaded_image),
                    organization::description.eq(aptocracy_data.description),
                ))
                .get_result(conn);
        } else {
            response = diesel::update(organization::table)
                .filter(organization::address.eq(aptocracy_data.aptocracy_address))
                .set((organization::description.eq(aptocracy_data.description),))
                .get_result(conn);
        }

        parse_graphql_response::<Aptocracy, AptocracyDto>(response)
    }
}

#[derive(GraphQLObject)]
pub struct RoleConfig {
    pub name: String,
    pub actions: Vec<i32>,
    pub role_weight: f64,
}
#[derive(GraphQLObject)]
pub struct GoverningCollection {
    pub creator: String,
    pub name: String,
}

#[derive(GraphQLObject)]
pub struct AptocracyDto {
    pub address: String,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub creator: String,
    pub default_role: String,
    pub governing_coin: String,
    pub governing_collection_info: Option<GoverningCollection>,
    pub invite_only: bool,
    pub main_governance: Option<f64>,
    pub max_voter_weight: Option<f64>,
    pub org_type: String,
    pub treasury_count: i32,
    pub role_config: Vec<RoleConfig>,
    pub created_at: NaiveDateTime,
    pub main_treasury: Option<String>,
}

impl AptocracyParser<AptocracyDto> for Aptocracy {
    fn from(self) -> AptocracyDto {
        AptocracyDto {
            address: self.address,
            name: self.name,
            image: self.image,
            description: self.description,
            creator: self.creator,
            default_role: self.default_role,
            governing_coin: self.governing_coin,
            governing_collection_info: map_to_governing_collection_info(
                self.governing_collection_info,
            ),
            invite_only: self.invite_only,
            main_governance: map_to_f64(self.main_governance),
            max_voter_weight: map_to_f64(self.max_voter_weight),
            org_type: self.org_type,
            treasury_count: self.treasury_count,
            role_config: map_to_role_config(self.role_config),
            created_at: self.created_at,
            main_treasury: self.main_treasury,
        }
    }
}

impl AptocracyParser<Vec<AptocracyDto>> for Vec<Aptocracy> {
    fn from(self) -> Vec<AptocracyDto> {
        self.iter().map(|item| item.clone().from()).collect()
    }
}

#[derive(GraphQLInputObject)]
pub struct UpdateAptocracy {
    pub aptocracy_address: String,
    pub image_base_64: String,
    pub description: String,
}
