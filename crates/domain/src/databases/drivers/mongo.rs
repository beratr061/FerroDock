use super::{Credentials, DatabaseDriver, RestoreMethod};

const AUTH: &str = "--username \"$MONGO_INITDB_ROOT_USERNAME\" --password \"$MONGO_INITDB_ROOT_PASSWORD\" --authenticationDatabase admin";

pub struct Mongo;

impl DatabaseDriver for Mongo {
    fn engine(&self) -> &'static str {
        "mongo"
    }

    fn image(&self) -> &'static str {
        "mongo"
    }

    fn versions(&self) -> &'static [&'static str] {
        &["8.0", "7.0"]
    }

    fn port(&self) -> u16 {
        27017
    }

    fn data_dir(&self, _version: &str) -> &'static str {
        "/data/db"
    }

    fn env(&self, credentials: &Credentials<'_>) -> Vec<String> {
        vec![
            format!("MONGO_INITDB_DATABASE={}", credentials.database),
            format!("MONGO_INITDB_ROOT_USERNAME={}", credentials.username),
            format!("MONGO_INITDB_ROOT_PASSWORD={}", credentials.password),
        ]
    }

    fn healthcheck(&self) -> &'static str {
        "mongosh --quiet --eval \"db.adminCommand('ping').ok\" | grep -q 1"
    }

    fn scheme(&self) -> &'static str {
        "mongodb"
    }

    fn dump_script(&self) -> String {
        format!("exec mongodump --archive --gzip --quiet {AUTH}")
    }

    fn restore_method(&self) -> RestoreMethod {
        RestoreMethod::Stream(format!(
            "exec mongorestore --archive --gzip --drop --quiet {AUTH}"
        ))
    }

    fn dump_extension(&self) -> &'static str {
        "archive.gz"
    }

    fn url_suffix(&self, database: &str) -> String {
        format!("/{database}?authSource=admin")
    }
}
