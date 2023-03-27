# Intro
[`sansan`](https://github.com/hinto-janai/sansan) is a cross-platform (Windows/macOS/Linux) real-time music engine.

Put simply:
- `sansan` plays audio
- You control what/when/how it plays
- It tries very hard to make sure audio is playing when it should be

Although `sansan` could be used as general purpose audio engine, there are other better options.

`sansan` is focused specifically on music playback (single global concurrent queue, previous/forward signals, etc) and doesn't include things general audio engines would have (audio sample mutation, multi-audio mixing, etc).

An example of where `sansan` is used is [`Festival`](https://github.com/hinto-janai/festival). It uses `sansan` internally to handle all music playback tasks.
