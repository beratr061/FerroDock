use std::str::FromStr;

use ferrodock_core::{AppError, AppResult, Id};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Viewer,
    Developer,
    Admin,
    Owner,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Developer => "developer",
            Self::Admin => "admin",
            Self::Owner => "owner",
        }
    }
}

impl FromStr for Role {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "viewer" => Ok(Self::Viewer),
            "developer" => Ok(Self::Developer),
            "admin" => Ok(Self::Admin),
            "owner" => Ok(Self::Owner),
            _ => Err(AppError::validation("ROLE_INVALID").with_param("role", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Principal {
    pub user_id: Id,
    pub organization_id: Id,
    pub role: Role,
    pub session_id: Option<Id>,
    pub api_key_id: Option<Id>,
}

impl Principal {
    pub fn require(&self, minimum: Role) -> AppResult<()> {
        if self.role >= minimum {
            Ok(())
        } else {
            Err(AppError::forbidden())
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Membership {
    pub organization_id: Id,
    pub organization_name: String,
    pub organization_slug: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentUser {
    pub id: Id,
    pub email: String,
    pub name: String,
    pub locale: String,
    pub theme: String,
    pub active_organization_id: Id,
    pub memberships: Vec<Membership>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_ordering_controls_access() {
        let principal = Principal {
            user_id: Id::new(),
            organization_id: Id::new(),
            role: Role::Developer,
            session_id: None,
            api_key_id: None,
        };
        assert!(principal.require(Role::Viewer).is_ok());
        assert!(principal.require(Role::Developer).is_ok());
        assert!(principal.require(Role::Admin).is_err());
    }
}
