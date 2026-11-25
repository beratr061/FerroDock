use ferrodock_domain::auth::{CurrentUser, Membership, Role};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::Password;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoleDto {
    Viewer,
    Developer,
    Admin,
    Owner,
}

impl From<Role> for RoleDto {
    fn from(role: Role) -> Self {
        match role {
            Role::Viewer => Self::Viewer,
            Role::Developer => Self::Developer,
            Role::Admin => Self::Admin,
            Role::Owner => Self::Owner,
        }
    }
}

impl From<RoleDto> for Role {
    fn from(role: RoleDto) -> Self {
        match role {
            RoleDto::Viewer => Self::Viewer,
            RoleDto::Developer => Self::Developer,
            RoleDto::Admin => Self::Admin,
            RoleDto::Owner => Self::Owner,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub database: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SetupStatus {
    pub required: bool,
    pub token_required: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetupRequest {
    pub email: String,
    pub name: String,
    #[schema(value_type = String, format = Password)]
    pub password: Password,
    pub organization_name: String,
    pub locale: Option<String>,
    #[schema(value_type = Option<String>, format = Password)]
    pub setup_token: Option<Password>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SignInRequest {
    pub email: String,
    #[schema(value_type = String, format = Password)]
    pub password: Password,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MembershipDto {
    pub organization_id: Uuid,
    pub organization_name: String,
    pub organization_slug: String,
    pub role: RoleDto,
}

impl From<Membership> for MembershipDto {
    fn from(membership: Membership) -> Self {
        Self {
            organization_id: membership.organization_id.as_uuid(),
            organization_name: membership.organization_name,
            organization_slug: membership.organization_slug,
            role: membership.role.into(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub locale: String,
    pub theme: String,
    pub active_organization_id: Uuid,
    pub memberships: Vec<MembershipDto>,
}

impl From<CurrentUser> for MeResponse {
    fn from(user: CurrentUser) -> Self {
        Self {
            id: user.id.as_uuid(),
            email: user.email,
            name: user.name,
            locale: user.locale,
            theme: user.theme,
            active_organization_id: user.active_organization_id.as_uuid(),
            memberships: user
                .memberships
                .into_iter()
                .map(MembershipDto::from)
                .collect(),
        }
    }
}
