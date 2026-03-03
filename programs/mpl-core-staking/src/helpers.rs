use crate::constants::{MARKET_CLOSE_TIME, MARKET_OPEN_CLOSE_MARGIN, MARKET_OPEN_TIME, SECONDS_IN_A_DAY};

// Helpers
pub fn is_market_open(unix_timestamp: i64) -> bool {
    let seconds_since_midnight = unix_timestamp % SECONDS_IN_A_DAY;
    let weekday = (unix_timestamp / SECONDS_IN_A_DAY + 4) % 7;

    // Check if it's a weekday (Monday = 0, ..., Friday = 4)
    if weekday >= 5 {
        return false;
    }

    // Check if current time is within market hours
    seconds_since_midnight >= MARKET_OPEN_TIME && seconds_since_midnight < MARKET_CLOSE_TIME
}


pub fn is_within_15_minutes_of_market_open_or_close(unix_timestamp: i64) -> bool {
    let seconds_since_midnight = unix_timestamp % SECONDS_IN_A_DAY;

    // Check if current time is within 15 minutes after market open or within 15 minutes after market close
    (seconds_since_midnight >= MARKET_OPEN_TIME && seconds_since_midnight < MARKET_OPEN_TIME + MARKET_OPEN_CLOSE_MARGIN) ||
    (seconds_since_midnight >= MARKET_CLOSE_TIME && seconds_since_midnight < MARKET_CLOSE_TIME + MARKET_OPEN_CLOSE_MARGIN)
}