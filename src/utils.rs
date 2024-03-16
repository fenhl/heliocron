#[cfg(test)]
mod tests {
    #[cfg(feature = "integration-test")]
    #[tokio::test]
    async fn test_wait() {
        use chrono::{FixedOffset, TimeZone};

        use super::*;

        // Some time improbably far in the future.
        let wait_until = FixedOffset::west(0).timestamp(9999999999, 0);
        wait(wait_until).await.unwrap();
    }
}
