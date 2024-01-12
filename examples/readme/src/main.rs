// This test is roughly the same as `sansan`'s root README.md.

use sansan::{
	Engine,
	config::InitConfig,
	signal::{Add, Remove},
	state::{AudioStateSnapshot, AudioStateReader},
	source::Source,
};
use std::path::Path;

fn main() {
	//-------------------------------------------------------------------- Audio source set-up
	// Create a music engine with default settings.
	let mut engine = Engine::<()>::init(InitConfig::DEFAULT).unwrap();

	// Our music data (Paths and/or raw bytes)
	let track_1: &Path = Path::new("../../assets/audio/moonlight_sonata.mp3");
	let track_2: &[u8] = include_bytes!("../../../assets/audio/moonlight_sonata.mp3");

	// Convert them into audio `Source`'s that `sansan` can use.
	let track_1: Source<()> = Source::from(track_1);
	let track_2: Source<()> = Source::from(track_2);

	// Add the music to the back of the queue.
	engine.add(Add::from(track_1));
	engine.add(Add::from(track_2));

	// Start playing.
	engine.play();

	//-------------------------------------------------------------------- Non-blocking state reading
	// Our music is now playing
	// on the default audio device.
	//
	// We can mutate/view the queue
	// and adjust settings freely without
	// blocking the real-time audio thread.
	std::thread::sleep(std::time::Duration::from_secs(1));

	// For example, let's take a look at the current audio state.
	//
	// We can hold onto this data for all of eternity
	// and it will not block [sansan] from progressing.
	let reader: AudioStateReader<()> = engine.reader();
	let audio_state: AudioStateSnapshot<()> = reader.get();
	assert_eq!(audio_state.queue.len(), 2);
	assert_eq!(audio_state.playing, true);

	// We can view live playback information.
	// Some time has elapsed in the current track.
	assert!(audio_state.current.as_ref().unwrap().elapsed > 0.0);

	//-------------------------------------------------------------------- Non-blocking state writing
	// We can mutate the queue, without
	// blocking the real-time audio thread.
	//
	// Let's remove the 2nd track in the queue.
	let response: AudioStateSnapshot<()> = engine.remove(Remove { index: 1 }).unwrap();
	// Now the queue is only 1 length.
	assert_eq!(response.queue.len(), 1);

	// Skip to the next song.
	let response: AudioStateSnapshot<()> = engine.next();

	// Since there was only 1 track left,
	// and no repeat mode was on,
	// the [Engine] has stopped playing.
	assert_eq!(response.playing, false);
	assert_eq!(response.queue.len(), 0);
}