//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	valid_data::ValidData,
	macros::{recv,try_send,debug2,info2},
	state::{
		AudioStateSnapshot,
		AudioStateReader,
		AudioState,
		AtomicState,
	},
	config::{
		InitConfig,
		Callbacks,
		LiveConfig
	},
	actor::{
		audio::{Audio,AUDIO_BUFFER_LEN},
		decode::Decode,
		kernel::Kernel,
		pool::Pool,
		gc::Gc,
		caller::Caller,
	},
	signal::{
		Add,AddMany,Back,Clear,Previous,RemoveRange,Remove,
		Repeat,Seek,SetIndex,Shuffle,Skip,Volume,AddMethod,
		SeekError,Next,PreviousError,SkipError,
		BackError,SetIndexError,RemoveError, BackThreshold,
	}
};
use crossbeam::channel::{bounded,unbounded};
use std::sync::{
	Arc,
	Barrier,
	atomic::AtomicBool,
};

// Prevent collision with [S] generic.
use crossbeam::channel::Sender as S;
use crossbeam::channel::Receiver as R;

// Audio I/O backend.
use crate::output::AudioOutputStruct;

// Resampler backend.
use crate::resampler::ResamplerStruct;

//---------------------------------------------------------------------------------------------------- Engine
/// The main handle to `sansan`'s audio system.
///
/// TODO
#[allow(clippy::missing_docs_in_private_items)]
#[derive(Debug)]
pub struct Engine<Data>
where
	Data: ValidData,
{
	/// Data and objects.
	pub(super) reader: AudioStateReader<Data>,
	pub(super) config: LiveConfig,
	pub(super) atomic_state: Arc<AtomicState>,

	// Internal mirrored state.
	pub(super) repeat: Repeat,
	pub(super) volume: Volume,

	/// Signal to [Kernel] to tell all of our internal
	/// actors (threads) to start shutting down.
	pub(super) shutdown: S<()>,
	/// Same as above, but for [shutdown_hang()].
	pub(super) shutdown_hang: S<()>,
	/// [Kernel] telling us the shutdown
	/// process has been completed.
	pub(super) shutdown_done: R<()>,

	/// This channel is shared between all signals that don't
	/// have special output, i.e, they return `AudioStateSnapshot`.
	pub(super) recv_audio_state: R<AudioStateSnapshot<Data>>,

	/// Signals that have no input and output `AudioStateSnapshot`
	pub(super) send_toggle:   S<()>,
	pub(super) send_play:     S<()>,
	pub(super) send_pause:    S<()>,
	pub(super) send_next:     S<()>,
	pub(super) send_previous: S<()>,
	pub(super) send_stop:     S<()>,

	/// Signals that have input and output `AudioStateSnapshot`.
	pub(super) send_add:       S<Add<Data>>,
	pub(super) send_add_many:  S<AddMany<Data>>,
	pub(super) send_clear:     S<Clear>,
	pub(super) send_restore:   S<AudioState<Data>>,
	pub(super) send_repeat:    S<Repeat>,
	pub(super) send_volume:    S<Volume>,
	pub(super) send_shuffle:   S<Shuffle>,

	/// Signals that return `Result<T, E>`
	/// These don't use the common `recv_audio_state_snapshot`,
	/// as they return unique values.
	pub(super) send_seek:         S<Seek>,
	pub(super) recv_seek:         R<Result<AudioStateSnapshot<Data>, SeekError>>,
	pub(super) send_skip:         S<Skip>,
	pub(super) recv_skip:         R<Result<AudioStateSnapshot<Data>, SkipError>>,
	pub(super) send_back:         S<Back>,
	pub(super) recv_back:         R<Result<AudioStateSnapshot<Data>, BackError>>,
	pub(super) send_set_index:    S<SetIndex>,
	pub(super) recv_set_index:    R<Result<AudioStateSnapshot<Data>, SetIndexError>>,
	pub(super) send_remove:       S<Remove>,
	pub(super) recv_remove:       R<Result<AudioStateSnapshot<Data>, RemoveError>>,
	pub(super) send_remove_range: S<RemoveRange>,
	pub(super) recv_remove_range: R<Result<AudioStateSnapshot<Data>, RemoveError>>,
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {}