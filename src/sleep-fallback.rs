use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum Error {}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

type Result<T> = std::result::Result<T, Error>;

pub async fn sleep_until<Tz: chrono::TimeZone>(time: DateTime<Tz>) -> Result<()> {
    if let Ok(duration) = (time.with_timezone(&Utc) - Utc::now()).to_std() {
        tokio::time::sleep(duration).await;
    } else {
        // negative duration, do nothing
    }
    Ok(())
}
