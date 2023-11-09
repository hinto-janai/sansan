# Intro
[`sansan`](https://github.com/hinto-janai/sansan) is a cross-platform (Windows/macOS/Linux) real-time music engine.

Put simply:
- `sansan` plays music
- You control what/when/how it plays
- You can read/write the audio state without blocking
- It tries very hard to make sure audio is playing when it should be

Although `sansan` could be used as a general purpose audio engine, there are other better options.

`sansan` is focused specifically on music playback (single queue, previous/forward signals, etc) and doesn't include things general audio engines would have (audio sample mutation, mixing, multiple tracks, etc).

An example of where `sansan` is used is [`Festival`](https://github.com/hinto-janai/festival). It uses `sansan` internally to handle all music playback tasks.
