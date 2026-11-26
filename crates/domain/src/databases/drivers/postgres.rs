use super::{Credentials, DatabaseDriver};

pub struct Postgres;

impl DatabaseDriver for Postgres {
    fn engine(&self) -> &'static str {
        "postgres"
    }

    fn image(&self) -> &'static str {
        "postgres"
    }

    fn versions(&self) -> &'static [&'static str] {
        &["18", "17", "16", "15"]
    }

    fn port(&self) -> u16 {
        5432
    }

    fn data_dir(&self, version: &str) -> &'static str {
        let major: u32 = version.parse().unwrap_or(0);
        if major >= 18 {
            "/var/lib/postgresql"
        } else {
            "/var/lib/postgresql/data"
        }
    }

    fn env(&self, credentials: &Credentials<'_>) -> Vec<String> {
        vec![
            format!("POSTGRES_DB={}", credentials.database),
            format!("POSTGRES_USER={}", credentials.username),
            format!("POSTGRES_PASSWORD={}", credentials.password),
        ]
    }

    fn healthcheck(&self) -> &'static str {
        "pg_isready -h 127.0.0.1 -U \"$POSTGRES_USER\" -d \"$POSTGRES_DB\""
    }

    fn scheme(&self) -> &'static str {
        "postgresql"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_18_keeps_data_one_level_up() {
        assert_eq!(Postgres.data_dir("18"), "/var/lib/postgresql");
        assert_eq!(Postgres.data_dir("17"), "/var/lib/postgresql/data");
    }
}
