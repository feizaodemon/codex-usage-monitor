//! Convert server retry hints into bounded Windows timer delays.
use std::time::SystemTime;

pub fn delay_ms(value: &str, now: SystemTime) -> Option<u32> {
    let value = value.trim();
    let millis = if !value.is_empty() && value.bytes().all(|c| c.is_ascii_digit()) {
        value.parse::<u64>().ok()?.saturating_mul(1000)
    } else {
        httpdate::parse_http_date(value)
            .ok()?
            .duration_since(now)
            .unwrap_or_default()
            .as_millis()
            .min(u64::MAX as u128) as u64
    };
    Some(millis.clamp(1000, u32::MAX as u64) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn parses_seconds_dates_and_rejects_invalid_hints() {
        let date = httpdate::parse_http_date("Wed, 21 Oct 2015 07:28:00 GMT").unwrap();
        assert_eq!(delay_ms(" 120 ", UNIX_EPOCH), Some(120_000));
        assert_eq!(
            delay_ms(
                "Wed, 21 Oct 2015 07:28:00 GMT",
                date - Duration::from_secs(90)
            ),
            Some(90_000)
        );
        assert_eq!(delay_ms("Wed, 21 Oct 2015 07:28:00 GMT", date), Some(1000));
        assert_eq!(delay_ms("0", date), Some(1000));
        assert_eq!(delay_ms("18446744073709551615", date), Some(u32::MAX));
        for invalid in ["", "-1", "1.5", "tomorrow", "999999999999999999999999999"] {
            assert_eq!(delay_ms(invalid, date), None);
        }
    }
}
