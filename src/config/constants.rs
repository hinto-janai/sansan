//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::time::Duration;

//---------------------------------------------------------------------------------------------------- Constants
/// TODO
pub(crate) const DEFAULT_BACK_THRESHOLD: Duration = Duration::from_secs(DEFAULT_BACK_THRESHOLD_F32 as u64);
/// TODO
pub(crate) const DEFAULT_BACK_THRESHOLD_F32: f32 = 3.0;

/// TODO
pub(crate) const DEFAULT_ELAPSED_REFRESH_RATE: Duration = Duration::from_millis((DEFAULT_ELAPSED_REFRESH_RATE_F32 * 1000.0) as u64);
/// TODO
pub(crate) const DEFAULT_ELAPSED_REFRESH_RATE_F32: f32 = 0.033;

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn default_back_threshold() {
		assert_eq!(DEFAULT_BACK_THRESHOLD.as_secs_f32(), 3.0);
		assert_eq!(DEFAULT_BACK_THRESHOLD.as_secs_f32(), DEFAULT_BACK_THRESHOLD_F32);
	}

	#[test]
	fn default_elapsed_refresh_rate() {
		assert_eq!(DEFAULT_ELAPSED_REFRESH_RATE.as_secs_f32(), 0.033);
		assert_eq!(DEFAULT_ELAPSED_REFRESH_RATE.as_secs_f32(), DEFAULT_ELAPSED_REFRESH_RATE_F32);
	}
}
