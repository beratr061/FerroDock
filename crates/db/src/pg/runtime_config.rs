use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::runtime_config::{MountRecord, PortRecord, RuntimeConfigRecord, RuntimeConfigRepo};

struct PortRow {
    published_port: i32,
    target_port: i32,
    protocol: String,
}

fn port(row: PortRow) -> AppResult<PortRecord> {
    Ok(PortRecord {
        published: u16::try_from(row.published_port).map_err(AppError::internal)?,
        target: u16::try_from(row.target_port).map_err(AppError::internal)?,
        protocol: row.protocol,
    })
}

async fn insert_mounts(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    mounts: Vec<MountRecord>,
) -> AppResult<()> {
    for mount in mounts {
        sqlx::query!(
            "INSERT INTO application_mounts (application_id, target, kind, source, read_only)
             VALUES ($1, $2, $3, $4, $5)",
            id,
            mount.target,
            mount.kind,
            mount.source,
            mount.read_only
        )
        .execute(&mut **tx)
        .await
        .map_err(db_error)?;
    }
    Ok(())
}

async fn insert_ports(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    ports: Vec<PortRecord>,
) -> AppResult<()> {
    for port in ports {
        sqlx::query!(
            "INSERT INTO application_ports (application_id, published_port, target_port, protocol)
             VALUES ($1, $2, $3, $4)",
            id,
            i32::from(port.published),
            i32::from(port.target),
            port.protocol
        )
        .execute(&mut **tx)
        .await
        .map_err(db_error)?;
    }
    Ok(())
}

#[async_trait]
impl RuntimeConfigRepo for PgStore {
    async fn runtime_config(&self, application_id: Id) -> AppResult<RuntimeConfigRecord> {
        let id = application_id.as_uuid();
        let mounts = sqlx::query_as!(
            MountRecord,
            "SELECT kind, source, target, read_only FROM application_mounts
             WHERE application_id = $1 ORDER BY target",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        let ports = sqlx::query_as!(
            PortRow,
            "SELECT published_port, target_port, protocol FROM application_ports
             WHERE application_id = $1 ORDER BY published_port, protocol",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        let ports = ports.into_iter().map(port).collect::<AppResult<_>>()?;
        Ok(RuntimeConfigRecord { mounts, ports })
    }

    async fn replace_runtime_config(
        &self,
        application_id: Id,
        config: RuntimeConfigRecord,
    ) -> AppResult<()> {
        let id = application_id.as_uuid();
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        sqlx::query!("SELECT id FROM applications WHERE id = $1 FOR UPDATE", id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?;
        sqlx::query!(
            "DELETE FROM application_mounts WHERE application_id = $1",
            id
        )
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        sqlx::query!(
            "DELETE FROM application_ports WHERE application_id = $1",
            id
        )
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        insert_mounts(&mut tx, id, config.mounts).await?;
        insert_ports(&mut tx, id, config.ports).await?;
        tx.commit().await.map_err(db_error)
    }
}
