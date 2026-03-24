/// Calls `f` up to `max_attempts` times total, sleeping `interval` between failures.
/// Returns `Ok` on the first success, or the last `Err` if all attempts fail.
#[inline]
pub(super) fn retry_on_interval<F, T, E>(
    mut f: F,
    interval: std::time::Duration,
    max_attempts: u8,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    for _ in 1..max_attempts {
        match f() {
            Ok(result) => return Ok(result),
            Err(_) => std::thread::sleep(interval),
        }
    }
    f()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn succeeds_on_first_try() {
        let mut calls = 0u32;
        let result: Result<i32, &str> =
            retry_on_interval(|| { calls += 1; Ok(42) }, Duration::from_millis(1), 5);
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls, 1);
    }

    #[test]
    fn retries_then_succeeds() {
        let mut calls = 0;
        let result: Result<i32, &str> = retry_on_interval(
            || {
                calls += 1;
                if calls < 3 { Err("not yet") } else { Ok(calls as i32) }
            },
            Duration::from_millis(1),
            5,
        );
        assert_eq!(result.unwrap(), 3);
        assert_eq!(calls, 3);
    }

    #[test]
    fn all_fail_returns_last_error() {
        let mut calls = 0;
        let result: Result<i32, u32> =
            retry_on_interval(|| { calls += 1; Err(calls) }, Duration::from_millis(1), 3);
        assert_eq!(result.unwrap_err(), 3);
        assert_eq!(calls, 3);
    }

    #[test]
    fn max_attempts_one_makes_single_call() {
        let mut calls = 0;
        let result: Result<(), &str> =
            retry_on_interval(|| { calls += 1; Err("fail") }, Duration::from_millis(1), 1);
        assert_eq!(calls, 1);
        assert_eq!(result.unwrap_err(), "fail");
    }
}
