<div align="center">

![CI](https://github.com/hinto-janai/sansan/actions/workflows/ci.yml/badge.svg) [![crates.io](https://img.shields.io/crates/v/sansan.svg)](https://crates.io/crates/sansan) [![docs.rs](https://docs.rs/sansan/badge.svg)](https://docs.rs/sansan)

<img src="https://github.com/hinto-janai/sansan/assets/101352116/5c77d0aa-2f9b-4579-8d3c-b9e00e225179" width="60%"/>

</div>

## About
`sansan` is a real-time music playback library.

This library is for:
- Queue-based, real-time music playback
- Live audio state reading/writing without blocking
- Audio metadata reading (`ID3/Vorbis` tags)
- OS media control integration

This library is not:
- A general purpose audio playback library

`sansan` is built with music players in-mind - it is meant to be the part
of the system that handles the real-time audio decoding/playback,
while exposing an audio state read/write API that is non-blocking.

Although `sansan` _can_ be used for general purpose audio playback,
it does not include general purpose audio APIs like mixing, filters, multiple tracks, etc.

## Documentation
The book at [`https://sansan.cat`](https://sansan.cat) is the main user documentation. It holds all the information needed to get started with `sansan` - what things there are, what they do, how to use them.

The library documentation at [`https://docs.rs/sansan`](https://docs.rs/sansan) is the API reference, documenting inputs and outputs and other note-worthy things about the API, although, it does not contain detailed commentary on usage, how things work together, etc.

## Example
For more example usage of `sansan`, see [`examples/`](examples).

This example shows some basic usage of `sansan`:
- Creating the `Engine`
- Adding music to the queue
- Sending signals to the `Engine` (play, next)
- Reading/writing live audio state without blocking

```rust
// TODO
```

## Design
`sansan`'s abstract design is documented in [`DESIGN.md`](DESIGN.md).

This purpose of `DESIGN.md` is to act as a reference to allow for easier changes in the future.

Although, it mostly covers the system-wide view and does not include implementation details. For example, the real-time audio sample buffer - how big should it be?

These types of things are loosely defined in the code instead (with comments and reasoning) instead and within [`src/README.md`](src/README.md) - this document gives a more practical view on how `sansan` is organized, what files do what, where things are, why things do `x` instead of `y`, etc.

## Audio Dependencies
This table summarizes the **audio-specific** libraries used by `sansan` and their purpose.

| Dependency              | Owner                                                       | Purpose |
|-------------------------|-------------------------------------------------------------|---------|
| `audio_thread_priority` | [Mozilla](https://github.com/mozilla/audio_thread_priority) | Real-time audio thread promotion
| `cubeb`                 | [Mozilla](https://github.com/mozilla/cubeb-rs)              | Audio device output
| `cpal`                  | [RustAudio]https://github.com/rustaudio/cpal)               | Audio device output
| `souvlaki`              | [Sinono3](https://github.com/Sinono3/souvlaki)              | OS media control interface
| `symphonia`             | [Pdeljanov](https://github.com/pdeljanov/Symphonia)         | Audio demuxing/decoding/metadata
| `rubato`                | [HEnquist](https://github.com/HEnquist/rubato)              | Audio resampling

## Supported Targets
Only 64-bit targets (`x86_64`, `ARM64`, etc) are supported.

32-bit targets may work but are not tested on.

- Windows (WASAPI)
- macOS (CoreAudio)
- Linux (PulseAudio)

## Supported Audio
`sansan` uses [`symphonia`](https://github.com/pdeljanov/Symphonia) for audio decoding & metadata.

The supported audio codecs are:

- `AAC-LC`
- `ADPCM`
- `ALAC`
- `FLAC`
- `MP1/MP2/MP3`
- `Vorbis`
- `Opus`
- `WavPack`

The supported audio metadata formats are:

- `ID3v1`
- `ID3v2`
- `ISO/MP4`
- `RIFF`
- `Vorbis comment` (FLAC & OGG)

## MSRV
The `Minimum Supported Rust Version` is `1.70.0`.

## License
`sansan` is licensed under the [MIT License](https://github.com/hinto-janai/sansan/blob/main/LICENSE).

As of `v0.0.0`, `sansan`'s dependency tree includes the following licenses:
- `Apache-2.0`
- `BSD-2-Clause`
- `BSD-3-Clause`
- `ISC`
- `MIT`
- `MPL-2.0`
- `Unicode-DFS-2016`
