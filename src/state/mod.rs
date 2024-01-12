//! Audio state

mod constants;
pub use constants::BACK_THRESHOLD;
pub(crate) use constants::QUEUE_LEN;

mod audio_state;
pub use audio_state::AudioState;

mod audio_state_reader;
pub use audio_state_reader::AudioStateReader;

mod audio_state_snapshot;
pub use audio_state_snapshot::AudioStateSnapshot;

mod atomic_state;
pub(crate) use atomic_state::AtomicState;

mod current;
pub use current::Current;
