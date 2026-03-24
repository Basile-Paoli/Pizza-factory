/// Retries a function on failure with a specified interval and maximum number of retries.
/// Returns the result of the function if it succeeds, or the last error if it fails after all retries.
#[inline]
pub(super) fn retry_on_interval<F, T, E>(
    mut f: F,
    interval: std::time::Duration,
    max_retries: u8,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    for _ in 1..max_retries {
        match f() {
            Ok(result) => return Ok(result),
            Err(_) => std::thread::sleep(interval),
        }
    }
    f()
}


