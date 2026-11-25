use std::collections::HashMap;
use std::sync::Arc;

use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::variables::{NewVariable, VariableRecord};
use ferrodock_db::{ApplicationRepo, VariableRepo};
use serde_json::json;
use time::OffsetDateTime;

use super::validate::{validate_keys, validate_value};
use crate::audit::{Actor, AuditLog};
use crate::auth::{Principal, Role};

#[derive(Debug, Clone)]
pub struct VariableInput {
    pub key: String,
    pub value: Option<String>,
    pub secret: bool,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub key: String,
    pub value: Option<String>,
    pub secret: bool,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct VariableService {
    variables: Arc<dyn VariableRepo>,
    applications: Arc<dyn ApplicationRepo>,
    secrets: SecretBox,
    audit: AuditLog,
}

impl std::fmt::Debug for VariableService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VariableService")
    }
}

impl VariableService {
    pub fn new(
        repos: (Arc<dyn VariableRepo>, Arc<dyn ApplicationRepo>),
        secrets: SecretBox,
        audit: AuditLog,
    ) -> Self {
        let (variables, applications) = repos;
        Self {
            variables,
            applications,
            secrets,
            audit,
        }
    }

    pub async fn list(
        &self,
        principal: &Principal,
        application_id: Id,
    ) -> AppResult<Vec<Variable>> {
        self.owned(principal, application_id).await?;
        let records = self.variables.list(application_id).await?;
        records
            .into_iter()
            .map(|record| self.present(record))
            .collect()
    }

    pub async fn replace(
        &self,
        actor: Actor<'_>,
        application_id: Id,
        inputs: Vec<VariableInput>,
    ) -> AppResult<Vec<Variable>> {
        actor.principal.require(Role::Developer)?;
        self.owned(actor.principal, application_id).await?;
        validate_keys(inputs.iter().map(|input| input.key.as_str()))?;
        let current: HashMap<String, VariableRecord> = self
            .variables
            .list(application_id)
            .await?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect();
        let rows = inputs
            .iter()
            .map(|input| self.seal(input, current.get(&input.key)))
            .collect::<AppResult<Vec<_>>>()?;
        self.variables.replace(application_id, rows).await?;
        let keys: Vec<&str> = inputs.iter().map(|input| input.key.as_str()).collect();
        self.audit
            .change(
                actor,
                "variables.updated",
                application_id,
                json!({ "keys": keys }),
            )
            .await;
        self.list(actor.principal, application_id).await
    }

    pub async fn reveal(
        &self,
        actor: Actor<'_>,
        application_id: Id,
        key: &str,
    ) -> AppResult<String> {
        actor.principal.require(Role::Developer)?;
        self.owned(actor.principal, application_id).await?;
        let record = self
            .variables
            .list(application_id)
            .await?
            .into_iter()
            .find(|record| record.key == key)
            .ok_or_else(|| AppError::not_found("VARIABLE_NOT_FOUND"))?;
        let value = self.open(&record.value_enc)?;
        self.audit
            .change(
                actor,
                "variables.revealed",
                application_id,
                json!({ "key": key }),
            )
            .await;
        Ok(value)
    }

    fn seal(
        &self,
        input: &VariableInput,
        existing: Option<&VariableRecord>,
    ) -> AppResult<NewVariable> {
        let value_enc = match (&input.value, existing) {
            (Some(value), _) => {
                validate_value(&input.key, value)?;
                self.secrets.encrypt(value.as_bytes())?
            }
            (None, Some(record)) => record.value_enc.clone(),
            (None, None) => {
                return Err(AppError::validation("VARIABLE_VALUE_REQUIRED")
                    .with_param("key", input.key.as_str()));
            }
        };
        Ok(NewVariable {
            key: input.key.clone(),
            value_enc,
            secret: input.secret,
        })
    }

    fn open(&self, sealed: &str) -> AppResult<String> {
        let bytes = self.secrets.decrypt(sealed)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| AppError::validation("SECRET_CORRUPTED"))
    }

    fn present(&self, record: VariableRecord) -> AppResult<Variable> {
        let value = if record.secret {
            None
        } else {
            Some(self.open(&record.value_enc)?)
        };
        Ok(Variable {
            key: record.key,
            value,
            secret: record.secret,
            updated_at: record.updated_at,
        })
    }

    async fn owned(&self, principal: &Principal, id: Id) -> AppResult<()> {
        self.applications
            .find(principal.organization_id, id)
            .await?
            .map(|_| ())
            .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))
    }
}
