//! The request's own deadline: the `grpc-timeout` header that `tonic::Request::set_timeout` sets.

use std::time::Duration;

use http::HeaderValue;

/// The timeout to arm for a call: the shorter of the client's timeout and the request's
/// `grpc-timeout`, as tonic's `Channel` does.
pub(super) fn effective(
    client_timeout: Option<Duration>,
    grpc_timeout: Option<&HeaderValue>,
) -> Option<Duration> {
    let deadline = grpc_timeout.and_then(parse);

    match (client_timeout, deadline) {
        (Some(client_timeout), Some(deadline)) => Some(client_timeout.min(deadline)),
        (client_timeout, deadline) => client_timeout.or(deadline),
    }
}

/// Parses a `grpc-timeout` value: at most 8 digits followed by a unit (`H`, `M`, `S`, `m`, `u`
/// or `n`), as gRPC's PROTOCOL-HTTP2.md specifies. Returns `None` for anything else.
fn parse(value: &HeaderValue) -> Option<Duration> {
    let value = value.to_str().ok()?;
    let (digits, unit) = value.split_at_checked(value.len().checked_sub(1)?)?;

    if digits.is_empty() || digits.len() > 8 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let amount: u64 = digits.parse().ok()?;

    match unit {
        "H" => Some(Duration::from_secs(amount * 60 * 60)),
        "M" => Some(Duration::from_secs(amount * 60)),
        "S" => Some(Duration::from_secs(amount)),
        "m" => Some(Duration::from_millis(amount)),
        "u" => Some(Duration::from_micros(amount)),
        "n" => Some(Duration::from_nanos(amount)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(value: &str) -> HeaderValue {
        HeaderValue::from_str(value).unwrap()
    }

    #[test]
    fn parses_every_unit() {
        assert_eq!(parse(&header("2H")), Some(Duration::from_secs(2 * 60 * 60)));
        assert_eq!(parse(&header("3M")), Some(Duration::from_secs(3 * 60)));
        assert_eq!(parse(&header("4S")), Some(Duration::from_secs(4)));
        assert_eq!(parse(&header("200m")), Some(Duration::from_millis(200)));
        assert_eq!(
            parse(&header("200000u")),
            Some(Duration::from_micros(200_000))
        );
        assert_eq!(
            parse(&header("99999999n")),
            Some(Duration::from_nanos(99_999_999))
        );
    }

    #[test]
    fn parses_what_tonic_sets() {
        let mut request = tonic::Request::new(());
        request.set_timeout(Duration::from_millis(200));

        let value = request.metadata().get("grpc-timeout").unwrap();
        let value = HeaderValue::from_str(value.to_str().unwrap()).unwrap();

        assert_eq!(parse(&value), Some(Duration::from_millis(200)));
    }

    #[test]
    fn rejects_malformed_values() {
        for value in ["", "S", "10", "10s", "-1S", "123456789S", "1.5S", "1 S"] {
            assert_eq!(parse(&header(value)), None, "{value:?}");
        }
    }

    #[test]
    fn uses_the_shorter_timeout() {
        let second = Some(Duration::from_secs(1));
        let deadline = header("200m");

        assert_eq!(effective(None, None), None);
        assert_eq!(effective(second, None), second);
        assert_eq!(
            effective(None, Some(&deadline)),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            effective(second, Some(&deadline)),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            effective(Some(Duration::from_millis(50)), Some(&deadline)),
            Some(Duration::from_millis(50))
        );
        assert_eq!(effective(second, Some(&header("soon"))), second);
    }
}
