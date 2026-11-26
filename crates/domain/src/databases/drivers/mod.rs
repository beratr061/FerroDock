mod key_value;
mod mongo;
mod mysql;
mod postgres;

use ferrodock_core::{AppError, AppResult};

use self::key_value::KeyValue;
use self::mongo::Mongo;
use self::mysql::MySqlFamily;
use self::postgres::Postgres;

#[derive(Debug, Clone, Copy)]
pub struct Credentials<'a> {
    pub database: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub root_password: Option<&'a str>,
}

pub trait DatabaseDriver: Send + Sync {
    fn engine(&self) -> &'static str;
    fn image(&self) -> &'static str;
    fn versions(&self) -> &'static [&'static str];
    fn port(&self) -> u16;
    fn data_dir(&self, version: &str) -> &'static str;
    fn env(&self, credentials: &Credentials<'_>) -> Vec<String>;
    fn healthcheck(&self) -> &'static str;
    fn scheme(&self) -> &'static str;

    fn command(&self, _credentials: &Credentials<'_>) -> Option<Vec<String>> {
        None
    }

    fn needs_root_password(&self) -> bool {
        false
    }

    fn fixed_username(&self) -> Option<&'static str> {
        None
    }

    fn url_suffix(&self, database: &str) -> String {
        format!("/{database}")
    }
}

static DRIVERS: [&dyn DatabaseDriver; 6] = [
    &Postgres,
    &MySqlFamily::MYSQL,
    &MySqlFamily::MARIADB,
    &Mongo,
    &KeyValue::REDIS,
    &KeyValue::VALKEY,
];

pub fn drivers() -> &'static [&'static dyn DatabaseDriver] {
    &DRIVERS
}

pub fn driver(engine: &str) -> AppResult<&'static dyn DatabaseDriver> {
    DRIVERS
        .iter()
        .copied()
        .find(|candidate| candidate.engine() == engine)
        .ok_or_else(|| AppError::validation("DATABASE_ENGINE_INVALID"))
}

pub fn connection_url(
    driver: &dyn DatabaseDriver,
    credentials: &Credentials<'_>,
    host: &str,
    port: u16,
) -> String {
    format!(
        "{}://{}:{}@{host}:{port}{}",
        driver.scheme(),
        credentials.username,
        credentials.password,
        driver.url_suffix(credentials.database)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const CREDENTIALS: Credentials<'static> = Credentials {
        database: "orders",
        username: "app",
        password: "secret",
        root_password: Some("root"),
    };

    fn url(engine: &str, port: u16) -> Option<String> {
        driver(engine)
            .ok()
            .map(|found| connection_url(found, &CREDENTIALS, "fd-abc", port))
    }

    #[test]
    fn every_engine_is_registered_once_with_a_default_version() {
        for registered in drivers() {
            assert_ne!(registered.versions().len(), 0);
            assert!(
                registered
                    .data_dir(registered.versions()[0])
                    .starts_with('/')
            );
            let found = driver(registered.engine()).map(DatabaseDriver::image);
            assert_eq!(found.ok(), Some(registered.image()));
        }
        assert!(driver("oracle").is_err());
    }

    #[test]
    fn builds_connection_urls() {
        assert_eq!(
            url("postgres", 5432).as_deref(),
            Some("postgresql://app:secret@fd-abc:5432/orders")
        );
        assert_eq!(
            url("mongo", 27017).as_deref(),
            Some("mongodb://app:secret@fd-abc:27017/orders?authSource=admin")
        );
        assert_eq!(
            url("redis", 6379).as_deref(),
            Some("redis://app:secret@fd-abc:6379")
        );
    }
}
