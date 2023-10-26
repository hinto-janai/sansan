//---------------------------------------------------------------------------------------------------- Use
use crate::api::audio_state::AudioState;

//---------------------------------------------------------------------------------------------------- Signal
#[derive(Debug,Clone)]
pub struct Signal;

//---------------------------------------------------------------------------------------------------- Signal Etc
struct Seek;
enum AudioSeekError {}
enum AudioNextError {}
enum AudioPreviousError {}
struct Skip;
struct Back;
struct Repeat;
struct Shuffle;
struct Volume;
struct QueueAdd;
enum AudioIndexError {}
struct RemoveRange;

//---------------------------------------------------------------------------------------------------- Signal Impl
impl Signal {
	fn toggle(&self) { todo!() }
	fn play(&self) { todo!() }
	fn pause(&self) { todo!() }
	fn seek(&self, seek: Seek, second: u64) -> Result<usize, AudioSeekError> { todo!() }
	fn next(&self) -> Result<usize, AudioNextError> { todo!() }
	fn previous(&self, threshold: Option<u32>) -> Result<usize, AudioPreviousError> { todo!() }
	fn skip(&self, skip: Skip) { todo!() } // defines behavior when error happens
	fn back(&self, back: Back) { todo!() }
	fn repeat(&self, repeat: Repeat) -> Repeat { todo!() }
	fn shuffle(&self, shuffle: Shuffle) -> Shuffle { todo!() }
	fn volume(&self, volume: Volume) -> Volume { todo!() }
	fn restore<QueueData: Clone>(audio_state: AudioState<QueueData>) -> Result<(), ()> { todo!() }
	fn add(&self, add: QueueAdd) { todo!() } // path: impl AsRef<Path>, append: Append, clear: bool, play: bool)
	fn clear(&self, keep_playing: bool) { todo!() }
	fn set_index(index: usize) -> Result<usize, AudioIndexError> { todo!() }
	fn remove_range(remove_range: RemoveRange) { todo!() } // defines what happens on included remove song, other errors, etc
}