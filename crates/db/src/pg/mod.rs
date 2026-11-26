mod api_keys;
mod application_rows;
mod applications;
mod audit;
mod backup_destinations;
mod backup_policies;
mod backup_runs;
mod databases;
mod deployments;
mod domains;
mod environments;
mod identity;
mod jobs;
mod login_attempts;
mod projects;
mod runtime_config;
mod sessions;
mod variables;

use std::time::Duration;

use ferrodock_core::{AppError, AppResult};
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

pub use jobs::JobSignals;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations/postgres");

#[derive(Debug, Clone)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    pub async fn connect(database_url: &str, max_connections: u32) -> AppResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(10))
            .connect(database_url)
            .await
            .map_err(|error| {
                AppError::new(
                    ferrodock_core::ErrorKind::Unavailable,
                    "DATABASE_UNAVAILABLE",
                )
                .with_source(error)
            })?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> AppResult<()> {
        MIGRATOR.run(&self.pool).await.map_err(AppError::internal)
    }

    pub async fn ping(&self) -> AppResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(db_error)
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}

pub(crate) fn db_error(error: sqlx::Error) -> AppError {
    let is_unique_violation = error
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation);
    if is_unique_violation {
        AppError::conflict("ALREADY_EXISTS").with_source(error)
    } else {
        AppError::internal(error)
    }
}
