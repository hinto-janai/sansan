//! TODO

//---------------------------------------------------------------------------------------------------- Constants
/// `QUEUE_LEN` should be the initial buffer size of the [`AudioState`]'s queue.
///
/// This should be big enough such a resize never
/// occurs (in most situations) but not too big incase
/// the generic [Data] the user provides is large.
pub(crate) const QUEUE_LEN: usize = 256;

/// TODO
pub const BACK_THRESHOLD: f64 = 3.0;