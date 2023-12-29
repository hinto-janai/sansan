//! TODO

mod kernel;
pub(crate) use kernel::{
	Kernel,
	DiscardCurrentAudio,
	Channels,
	InitArgs,
	KernelToDecode,
};

// Signal handlers.
mod toggle;
mod play;
mod pause;
mod stop;
mod clear;
mod restore;
mod shuffle;
mod repeat;
mod volume;
mod next;
mod previous;
mod add;
mod add_many;
mod seek;
mod skip;
mod back;
mod set_index;
mod remove;
mod remove_range;
