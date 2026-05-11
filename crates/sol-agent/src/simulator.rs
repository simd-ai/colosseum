use rand::Rng;

/// Simulate a CFD compute workload by sleeping for a random duration.
/// Returns the simulated execution duration in seconds.
pub async fn simulate_compute(gpu_class: &str, max_duration: u32) -> u32 {
    // Duration varies by GPU class (faster GPUs = shorter sim)
    let (min_sec, max_sec) = match gpu_class {
        "H100" => (3, (max_duration / 2).max(5)),
        "A100" => (5, (max_duration * 2 / 3).max(8)),
        "L40S" => (8, (max_duration * 3 / 4).max(12)),
        _      => (5, max_duration.max(10)),
    };

    // ThreadRng is !Send, so scope it tightly before the first .await.
    let duration = {
        let mut rng = rand::thread_rng();
        rng.gen_range(min_sec..=max_sec.min(max_duration))
    };

    tracing::info!(
        "⚙️  Simulating {} compute for {}s...",
        gpu_class,
        duration,
    );

    // Simulate with progress reporting every 5s
    let mut elapsed = 0u32;
    while elapsed < duration {
        let chunk = 5u32.min(duration - elapsed);
        tokio::time::sleep(tokio::time::Duration::from_secs(chunk as u64)).await;
        elapsed += chunk;
        let pct = (elapsed as f32 / duration as f32 * 100.0) as u32;
        tracing::debug!("  Progress: {}% ({}/{}s)", pct, elapsed, duration);
    }

    tracing::info!("✅ Compute simulation complete ({}s)", duration);
    duration
}
