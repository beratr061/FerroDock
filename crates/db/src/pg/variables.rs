use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};

use super::{PgStore, db_error};
use crate::variables::{NewVariable, VariableRecord, VariableRepo};

#[async_trait]
impl VariableRepo for PgStore {
    async fn list(&self, application_id: Id) -> AppResult<Vec<VariableRecord>> {
        sqlx::query_as!(
            VariableRecord,
            "SELECT key, value_enc, secret, updated_at FROM application_variables
             WHERE application_id = $1 ORDER BY key",
            application_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)
    }

    async fn replace(&self, application_id: Id, variables: Vec<NewVariable>) -> AppResult<()> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        sqlx::query!(
            "SELECT id FROM applications WHERE id = $1 FOR UPDATE",
            application_id.as_uuid()
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?;
        sqlx::query!(
            "DELETE FROM application_variables WHERE application_id = $1",
            application_id.as_uuid()
        )
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        for variable in variables {
            sqlx::query!(
                "INSERT INTO application_variables (application_id, key, value_enc, secret)
                 VALUES ($1, $2, $3, $4)",
                application_id.as_uuid(),
                variable.key,
                variable.value_enc,
                variable.secret
            )
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        }
        tx.commit().await.map_err(db_error)
    }
}
