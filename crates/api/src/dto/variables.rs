use ferrodock_domain::variables::{Variable, VariableInput};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct VariableDto {
    pub key: String,
    pub value: Option<String>,
    pub secret: bool,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VariableInputDto {
    pub key: String,
    pub value: Option<String>,
    pub secret: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReplaceVariablesRequest {
    pub variables: Vec<VariableInputDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RevealedVariableDto {
    pub value: String,
}

impl From<Variable> for VariableDto {
    fn from(variable: Variable) -> Self {
        Self {
            key: variable.key,
            value: variable.value,
            secret: variable.secret,
            updated_at: variable.updated_at,
        }
    }
}

impl From<VariableInputDto> for VariableInput {
    fn from(input: VariableInputDto) -> Self {
        Self {
            key: input.key,
            value: input.value,
            secret: input.secret,
        }
    }
}
