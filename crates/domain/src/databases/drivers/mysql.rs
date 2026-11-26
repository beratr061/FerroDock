use super::{Credentials, DatabaseDriver};

pub struct MySqlFamily {
    engine: &'static str,
    prefix: &'static str,
    versions: &'static [&'static str],
    healthcheck: &'static str,
}

impl MySqlFamily {
    pub const MYSQL: Self = Self {
        engine: "mysql",
        prefix: "MYSQL",
        versions: &["8.4", "9"],
        healthcheck: "mysqladmin ping -h 127.0.0.1 -uroot -p\"$MYSQL_ROOT_PASSWORD\" --silent",
    };

    pub const MARIADB: Self = Self {
        engine: "mariadb",
        prefix: "MARIADB",
        versions: &["11.8", "11.4", "10.11"],
        healthcheck: "healthcheck.sh --connect --innodb_initialized",
    };
}

impl DatabaseDriver for MySqlFamily {
    fn engine(&self) -> &'static str {
        self.engine
    }

    fn image(&self) -> &'static str {
        self.engine
    }

    fn versions(&self) -> &'static [&'static str] {
        self.versions
    }

    fn port(&self) -> u16 {
        3306
    }

    fn data_dir(&self, _version: &str) -> &'static str {
        "/var/lib/mysql"
    }

    fn env(&self, credentials: &Credentials<'_>) -> Vec<String> {
        let prefix = self.prefix;
        let root = credentials.root_password.unwrap_or(credentials.password);
        vec![
            format!("{prefix}_DATABASE={}", credentials.database),
            format!("{prefix}_USER={}", credentials.username),
            format!("{prefix}_PASSWORD={}", credentials.password),
            format!("{prefix}_ROOT_PASSWORD={root}"),
        ]
    }

    fn healthcheck(&self) -> &'static str {
        self.healthcheck
    }

    fn scheme(&self) -> &'static str {
        "mysql"
    }

    fn needs_root_password(&self) -> bool {
        true
    }
}
