use std::str::FromStr;

use croner::Cron;
use ferrodock_core::{AppError, AppResult};
use jiff::Timestamp;
use jiff::tz::TimeZone;
use time::OffsetDateTime;

const MAX_SCHEDULE_LEN: usize = 100;
const FIELDS: usize = 5;

fn invalid() -> AppError {
    AppError::validation("BACKUP_SCHEDULE_INVALID")
}

fn parse(schedule: &str) -> AppResult<Cron> {
    let trimmed = schedule.trim();
    let alias = trimmed.starts_with('@');
    let fields = trimmed.split_whitespace().count();
    if trimmed.len() > MAX_SCHEDULE_LEN || (!alias && fields != FIELDS) {
        return Err(invalid());
    }
    Cron::from_str(trimmed).map_err(|_| invalid())
}

pub fn normalize(schedule: &str) -> AppResult<String> {
    parse(schedule)?;
    Ok(schedule.split_whitespace().collect::<Vec<_>>().join(" "))
}

pub fn next_after(schedule: &str, after: OffsetDateTime) -> AppResult<OffsetDateTime> {
    let cron = parse(schedule)?;
    let start = Timestamp::from_second(after.unix_timestamp())
        .map_err(|_| invalid())?
        .to_zoned_datetime(TimeZone::UTC);
    let next = cron
        .find_next_occurrence(&start, false)
        .map_err(|_| invalid())?;
    OffsetDateTime::from_unix_timestamp(next.timestamp().as_second()).map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn computes_the_next_run_in_utc() -> AppResult<()> {
        let now = datetime!(2026-11-26 10:15 UTC);
        assert_eq!(
            next_after("0 3 * * *", now)?,
            datetime!(2026-11-27 03:00 UTC)
        );
        assert_eq!(next_after("@hourly", now)?, datetime!(2026-11-26 11:00 UTC));
        assert_eq!(normalize(" 30  2 * * 1 ")?, "30 2 * * 1");
        assert!(normalize("* * * * * *").is_err());
        assert!(normalize("61 * * * *").is_err());
        assert!(normalize("rm -rf /").is_err());
        Ok(())
    }
}
