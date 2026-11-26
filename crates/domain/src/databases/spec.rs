use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::databases::DatabaseRecord;
use ferrodock_infra::docker::{DatabaseSpec, network_alias, short_key};
use zeroize::Zeroizing;

use super::drivers::{Credentials, driver};

pub struct Secrets {
    pub password: Zeroizing<String>,
    pub root_password: Option<Zeroizing<String>>,
}

pub fn alias(id: Id) -> String {
    network_alias(id)
}

pub fn volume(id: Id) -> String {
    format!("fd-{}-data", short_key(id, 12))
}

fn container(record: &DatabaseRecord) -> String {
    format!("{}-{}", record.slug, short_key(record.id, 12))
}

fn open(secrets: &SecretBox, sealed: &str) -> AppResult<Zeroizing<String>> {
    let bytes = secrets.decrypt(sealed)?;
    String::from_utf8(bytes.to_vec())
        .map(Zeroizing::new)
        .map_err(|_| AppError::validation("SECRET_CORRUPTED"))
}

pub fn reveal(secrets: &SecretBox, record: &DatabaseRecord) -> AppResult<Secrets> {
    Ok(Secrets {
        password: open(secrets, &record.password_enc)?,
        root_password: record
            .root_password_enc
            .as_deref()
            .map(|sealed| open(secrets, sealed))
            .transpose()?,
    })
}

pub fn credentials<'a>(record: &'a DatabaseRecord, secrets: &'a Secrets) -> Credentials<'a> {
    Credentials {
        database: &record.database_name,
        username: &record.username,
        password: &secrets.password,
        root_password: secrets.root_password.as_deref().map(String::as_str),
    }
}

pub fn container_spec(record: &DatabaseRecord, secrets: &SecretBox) -> AppResult<DatabaseSpec> {
    let engine = driver(&record.engine)?;
    let opened = reveal(secrets, record)?;
    let login = credentials(record, &opened);
    Ok(DatabaseSpec {
        name: container(record),
        image: format!("{}:{}", engine.image(), record.version),
        database_id: record.id.to_string(),
        alias: alias(record.id),
        env: engine.env(&login),
        command: engine.command(),
        healthcheck: engine.healthcheck().to_owned(),
        volume: volume(record.id),
        data_dir: engine.data_dir(&record.version).to_owned(),
        port: engine.port(),
        published: record.external_port,
        memory_bytes: record
            .limits
            .memory_mb
            .map(|mb| i64::from(mb) * 1024 * 1024),
        nano_cpus: record
            .limits
            .cpu_millis
            .map(|millis| i64::from(millis) * 1_000_000),
    })
}
