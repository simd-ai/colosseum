/// Assignment strategies for job-to-provider matching.
///
/// Current strategy: Round-robin fairness (lowest total_jobs_completed first).
/// The query in sol-db already sorts by total_jobs_completed ASC,
/// so the first result is always the least-loaded provider.
///
/// Future strategies could include:
/// - Weighted scoring (GPU capability × availability × reputation)
/// - Geographic proximity
/// - Price bidding
pub struct AssignmentStrategy;

impl AssignmentStrategy {
    /// Calculate SCU cost for a job based on GPU class and duration.
    /// Fixed rate: 1 SCU = 1 GPU-second for demo purposes.
    pub fn calculate_scu(gpu_count: u8, duration_sec: u32) -> u64 {
        (gpu_count as u64) * (duration_sec as u64)
    }

    /// Calculate token payout from SCU amount.
    /// Rate: 1 SCU = 1000 tokens (with 6 decimals = 0.001 GRID per SCU).
    pub fn scu_to_tokens(scu_amount: u64) -> u64 {
        scu_amount.saturating_mul(1_000) // 0.001 GRID per SCU (6 decimal places)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scu_calculation() {
        assert_eq!(AssignmentStrategy::calculate_scu(4, 30), 120);
        assert_eq!(AssignmentStrategy::calculate_scu(1, 60), 60);
    }

    #[test]
    fn test_token_conversion() {
        assert_eq!(AssignmentStrategy::scu_to_tokens(100), 100_000);
        assert_eq!(AssignmentStrategy::scu_to_tokens(0), 0);
    }
}
