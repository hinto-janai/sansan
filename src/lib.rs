//! `sansan`
//!
//! Real-time audio engine.
//!
//! LP - 2019-09-05 - 1995
//! CD - 2019-09-18 - 19918

// Some notes.
//
// As of 2023-09-09 `std::time::Instant` uses:
// - Windows: `QueryPerformanceCounter`
// - macOS: `mach_absolute_time`
// - Linux: `clock_gettime`
//
// These should all be in the sub-millisecond range
// and should be okay to use as a timer in realtime-ish
// situations when dealing with audio, e.g:
// ```
// let timer = std::time::Instant::now();
//
// if timer.elapsed().as_secs_f32() < 0.015 {
//     // we don't have time, write audio samples to hardware
// } else {
//     // we have some time, do other stuff
// }
// ```
pub fn sansan() {
	println!("sansan");
}

//---------------------------------------------------------------------------------------------------- Lints

//---------------------------------------------------------------------------------------------------- Public API
mod audio_state;
pub use audio_state::{AudioStateReader,AudioState};

mod engine;
pub use engine::Engine;

mod source;
pub use source::{Source,SourcePath,SourceBytes};

mod channel;
pub use channel::{SansanReceiver,SansanSender};

///
pub mod config;

///
pub mod signal;

//---------------------------------------------------------------------------------------------------- Private Usage
mod audio;
mod internals;
mod patch;

//----------------------------------------------------------------------------------------------------
