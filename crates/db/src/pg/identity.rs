use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::IdentityRepo;
use crate::records::{MembershipRecord, NewOwner, OrganizationRecord, UserRecord};

const OWNER_BOOTSTRAP_LOCK: i64 = 0x00F3_D0C4;

#[async_trait]
impl IdentityRepo for PgStore {
    async fn has_users(&self) -> AppResult<bool> {
        sqlx::query_scalar!(r#"SELECT EXISTS (SELECT 1 FROM users) AS "exists!""#)
            .fetch_one(&self.pool)
            .await
            .map_err(db_error)
    }

    async fn create_owner(&self, owner: NewOwner) -> AppResult<(UserRecord, OrganizationRecord)> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        sqlx::query!("SELECT pg_advisory_xact_lock($1)", OWNER_BOOTSTRAP_LOCK)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        let exists = sqlx::query_scalar!(r#"SELECT EXISTS (SELECT 1 FROM users) AS "exists!""#)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;
        if exists {
            return Err(AppError::conflict("SETUP_ALREADY_COMPLETED"));
        }
        insert_owner_rows(&mut tx, &owner).await?;
        tx.commit().await.map_err(db_error)?;
        Ok(owner_records(owner))
    }

    async fn find_user_by_email(&self, email_normalized: &str) -> AppResult<Option<UserRecord>> {
        sqlx::query_as!(
            UserRow,
            "SELECT id, email, name, password_hash, locale, theme
             FROM users WHERE email_normalized = $1",
            email_normalized
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(UserRecord::from))
        .map_err(db_error)
    }

    async fn find_user(&self, id: Id) -> AppResult<Option<UserRecord>> {
        sqlx::query_as!(
            UserRow,
            "SELECT id, email, name, password_hash, locale, theme
             FROM users WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(UserRecord::from))
        .map_err(db_error)
    }

    async fn memberships(&self, user_id: Id) -> AppResult<Vec<MembershipRecord>> {
        let rows = sqlx::query!(
            "SELECT o.id, o.name, o.slug, m.role FROM org_members m
             JOIN organizations o ON o.id = m.organization_id
             WHERE m.user_id = $1 ORDER BY m.created_at",
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|r| MembershipRecord {
                organization: OrganizationRecord {
                    id: r.id.into(),
                    name: r.name,
                    slug: r.slug,
                },
                role: r.role,
            })
            .collect())
    }
}

fn owner_records(owner: NewOwner) -> (UserRecord, OrganizationRecord) {
    let user = UserRecord {
        id: owner.user_id,
        email: owner.email,
        name: owner.name,
        password_hash: owner.password_hash,
        locale: owner.locale,
        theme: "system".to_owned(),
    };
    let organization = OrganizationRecord {
        id: owner.organization_id,
        name: owner.organization_name,
        slug: owner.organization_slug,
    };
    (user, organization)
}

struct UserRow {
    id: Uuid,
    email: String,
    name: String,
    password_hash: String,
    locale: String,
    theme: String,
}

impl From<UserRow> for UserRecord {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id.into(),
            email: row.email,
            name: row.name,
            password_hash: row.password_hash,
            locale: row.locale,
            theme: row.theme,
        }
    }
}

async fn insert_owner_rows(tx: &mut Transaction<'_, Postgres>, owner: &NewOwner) -> AppResult<()> {
    sqlx::query!(
        "INSERT INTO organizations (id, name, slug) VALUES ($1, $2, $3)",
        owner.organization_id.as_uuid(),
        owner.organization_name,
        owner.organization_slug
    )
    .execute(&mut **tx)
    .await
    .map_err(db_error)?;
    sqlx::query!(
        "INSERT INTO users (id, email, email_normalized, name, password_hash, locale)
         VALUES ($1, $2, $3, $4, $5, $6)",
        owner.user_id.as_uuid(),
        owner.email,
        owner.email_normalized,
        owner.name,
        owner.password_hash,
        owner.locale
    )
    .execute(&mut **tx)
    .await
    .map_err(db_error)?;
    sqlx::query!(
        "INSERT INTO org_members (organization_id, user_id, role) VALUES ($1, $2, 'owner')",
        owner.organization_id.as_uuid(),
        owner.user_id.as_uuid()
    )
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(db_error)
}
