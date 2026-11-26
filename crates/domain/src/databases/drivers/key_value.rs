use super::{Credentials, DatabaseDriver};

pub struct KeyValue {
    engine: &'static str,
    image: &'static str,
    server: &'static str,
    versions: &'static [&'static str],
    healthcheck: &'static str,
}

impl KeyValue {
    pub const REDIS: Self = Self {
        engine: "redis",
        image: "redis",
        server: "redis-server",
        versions: &["8", "7.4"],
        healthcheck: "redis-cli -a \"$FERRODOCK_PASSWORD\" --no-auth-warning ping | grep -q PONG",
    };

    pub const VALKEY: Self = Self {
        engine: "valkey",
        image: "valkey/valkey",
        server: "valkey-server",
        versions: &["9", "8"],
        healthcheck: "valkey-cli -a \"$FERRODOCK_PASSWORD\" --no-auth-warning ping | grep -q PONG",
    };
}

impl DatabaseDriver for KeyValue {
    fn engine(&self) -> &'static str {
        self.engine
    }

    fn image(&self) -> &'static str {
        self.image
    }

    fn versions(&self) -> &'static [&'static str] {
        self.versions
    }

    fn port(&self) -> u16 {
        6379
    }

    fn data_dir(&self, _version: &str) -> &'static str {
        "/data"
    }

    fn env(&self, credentials: &Credentials<'_>) -> Vec<String> {
        vec![format!("FERRODOCK_PASSWORD={}", credentials.password)]
    }

    fn command(&self, credentials: &Credentials<'_>) -> Option<Vec<String>> {
        let args = [
            self.server,
            "--requirepass",
            credentials.password,
            "--appendonly",
            "yes",
        ];
        Some(args.map(str::to_owned).to_vec())
    }

    fn healthcheck(&self) -> &'static str {
        self.healthcheck
    }

    fn scheme(&self) -> &'static str {
        "redis"
    }

    fn fixed_username(&self) -> Option<&'static str> {
        Some("default")
    }

    fn url_suffix(&self, _database: &str) -> String {
        String::new()
    }
}
