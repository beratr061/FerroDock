use super::{Credentials, DatabaseDriver, RestoreMethod};

const STAGING: &str = "/data/ferrodock-restore.rdb";

pub struct KeyValue {
    engine: &'static str,
    image: &'static str,
    server: &'static str,
    cli: &'static str,
    versions: &'static [&'static str],
    healthcheck: &'static str,
}

impl KeyValue {
    pub const REDIS: Self = Self {
        engine: "redis",
        image: "redis",
        server: "redis-server",
        cli: "redis-cli",
        versions: &["8", "7.4"],
        healthcheck: "redis-cli -a \"$FERRODOCK_PASSWORD\" --no-auth-warning ping | grep -q PONG",
    };

    pub const VALKEY: Self = Self {
        engine: "valkey",
        image: "valkey/valkey",
        server: "valkey-server",
        cli: "valkey-cli",
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

    fn command(&self) -> Option<Vec<String>> {
        let server = self.server;
        let launch = format!(
            "exec docker-entrypoint.sh {server} --requirepass \"$FERRODOCK_PASSWORD\" --appendonly yes"
        );
        Some(vec!["sh".to_owned(), "-c".to_owned(), launch])
    }

    fn healthcheck(&self) -> &'static str {
        self.healthcheck
    }

    fn scheme(&self) -> &'static str {
        "redis"
    }

    fn dump_script(&self) -> String {
        let cli = self.cli;
        format!(
            "f=$(mktemp) && {cli} -a \"$FERRODOCK_PASSWORD\" --no-auth-warning --rdb \"$f\" >/dev/null && cat \"$f\"; s=$?; rm -f \"$f\"; exit $s"
        )
    }

    fn restore_method(&self) -> RestoreMethod {
        let (server, cli) = (self.server, self.cli);
        let sock = "-s /tmp/fd.sock";
        let prepare = [
            "set -e".to_owned(),
            format!("rm -rf /data/appendonlydir /data/appendonly.aof && mv {STAGING} /data/dump.rdb"),
            format!("{server} --dir /data --appendonly no --port 0 --unixsocket /tmp/fd.sock --daemonize yes"),
            format!("until {cli} {sock} ping 2>/dev/null | grep -q PONG; do sleep 0.2; done"),
            format!("{cli} {sock} config set appendonly yes >/dev/null"),
            format!("sleep 1; until {cli} {sock} info persistence | grep -q aof_rewrite_in_progress:0; do sleep 0.2; done"),
            format!("{cli} {sock} shutdown save || true"),
        ]
        .join("
");
        RestoreMethod::ReplaceDataFile {
            staging: STAGING,
            prepare,
        }
    }

    fn dump_extension(&self) -> &'static str {
        "rdb"
    }

    fn fixed_username(&self) -> Option<&'static str> {
        Some("default")
    }

    fn url_suffix(&self, _database: &str) -> String {
        String::new()
    }
}
