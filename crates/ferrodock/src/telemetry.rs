use tracing_subscriber::EnvFilter;

const DEFAULT_FILTER: &str = "info,tower_http=info,sqlx=warn";

pub fn init() {
    let json = std::env::var("FERRODOCK_LOG_JSON").map_or(true, |value| value != "false");
    let filter =
        EnvFilter::try_from_env("FERRODOCK_LOG").unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr);
    let result = if json {
        builder.json().try_init()
    } else {
        builder.compact().try_init()
    };
    if let Err(error) = result {
        tracing::warn!(%error, "tracing subscriber was already initialised");
    }
}
