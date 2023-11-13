//---------------------------------------------------------------------------------------------------- use
use someday::{Apply, ApplyReturn};
use std::collections::VecDeque;
use crate::audio_state::{AudioState,Track};
use crate::signal;

//---------------------------------------------------------------------------------------------------- PatchClear
// The [Patch]'s that can be applied to [AudioState] via [someday::Writer].
pub(crate) enum AudioStatePatch<TrackData: ValidTrackData> {
	// 1. [playing] boolean should be reversed under the condition
	//    that we are actually playing something, aka, [current] is [Some]
	Toggle,

	Play,

	// 1. [playing] boolean should be set to [false]
	Pause,

	Restore(AudioState<TrackData>),
	Repeat(signal::Repeat),
	Shuffle(signal::Shuffle),
	Volume(signal::Volume),

	// 1. The queue should be cleared
	// 2. All values should be set to null/0/etc
	// 3. [playing] boolean should be set based on user-input
	//
	// It should leave [AudioState] in a "new" empty state.
	Clear(signal::Clear),

	Add(signal::Add),
	Seek(signal::Seek),
	Next(signal::Next),
	Previous(signal::Previous),
	Skip(signal::Skip),
	Back(signal::Back),
	SetIndex(signal::SetIndex),
	RemoveRange(signal::RemoveRange),
	Remove(signal::Remove),
}

//---------------------------------------------------------------------------------------------------- Apply Impl
impl<TrackData: ValidTrackData> Apply<AudioStatePatch<TrackData>> for AudioState<TrackData> {
	#[inline]
	fn apply(patch: &mut AudioStatePatch<TrackData>, writer: &mut Self, _: &Self) {
		use AudioStatePatch as P;

		match patch {
			//---------------------------------------------------------------------------------------- Toggle
			P::Toggle => {
				// Guard against not playing and no track.
				if !writer.playing && writer.current.is_none() {
					return;
				}
				// XOR (flips boolean).
				writer.playing ^= true;
			}

			//---------------------------------------------------------------------------------------- Pause
			P::Pause => writer.playing = false,

			//---------------------------------------------------------------------------------------- Clear
			P::Clear(clear) => {
				writer.playing = clear.keep_playing;

				if !clear.keep_playing {
					writer.current = None;
				}

				writer.queue.clear();
			},

			_ => todo!(),
		}
	}
}