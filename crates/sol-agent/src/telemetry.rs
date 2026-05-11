use chrono::Utc;
use rand::Rng;
use uuid::Uuid;
use sol_common::GpuTelemetry;

/// Emit mock GPU telemetry data for a running job.
pub async fn emit_telemetry(
    provider_pubkey: &str,
    job_id: Uuid,
    gpu_count: u8,
    duration_sec: u32,
) {
    let mut rng = rand::thread_rng();

    for gpu_idx in 0..gpu_count {
        let telemetry = GpuTelemetry {
            provider_pubkey: provider_pubkey.to_string(),
            job_id,
            gpu_index: gpu_idx,
            utilization_pct: rng.gen_range(75.0..99.0),
            memory_used_mb: rng.gen_range(60_000..78_000),
            memory_total_mb: 81_920,
            temperature_c: rng.gen_range(55..82),
            power_draw_w: rng.gen_range(250..400),
            timestamp: Utc::now(),
        };

        tracing::info!(
            "📊 GPU[{}] util={:.1}% mem={}/{}MB temp={}°C power={}W",
            telemetry.gpu_index,
            telemetry.utilization_pct,
            telemetry.memory_used_mb,
            telemetry.memory_total_mb,
            telemetry.temperature_c,
            telemetry.power_draw_w,
        );
    }

    tracing::debug!(
        "Telemetry emitted for {} GPUs over {}s",
        gpu_count,
        duration_sec
    );
}
