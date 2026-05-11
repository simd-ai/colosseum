use anyhow::Result;
use std::future::Future;

/// Execute an async operation with exponential backoff retry.
///
/// - `max_retries`: Maximum number of retry attempts.
/// - `operation`: The async function to retry.
///
/// Backoff schedule: 500ms, 1s, 2s, 4s, 8s (exponential).
pub async fn with_retry<F, Fut, T>(max_retries: u32, operation: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut last_error = None;
    let mut delay_ms = 500u64;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    tracing::warn!(
                        "TX attempt {}/{} failed, retrying in {}ms...",
                        attempt + 1,
                        max_retries,
                        delay_ms
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    delay_ms = (delay_ms * 2).min(8_000); // Cap at 8 seconds
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown retry error")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_succeeds_eventually() {
        let counter = AtomicU32::new(0);
        let result = with_retry(3, || {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err(anyhow::anyhow!("not yet"))
                } else {
                    Ok("success")
                }
            }
        })
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
}
