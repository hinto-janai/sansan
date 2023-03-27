<div align="center">

# `sansan` (WIP)
Real-time music engine.

![CI](https://github.com/hinto-janai/sansan/actions/workflows/ci.yml/badge.svg) [![crates.io](https://img.shields.io/crates/v/sansan.svg)](https://crates.io/crates/sansan) [![docs.rs](https://docs.rs/sansan/badge.svg)](https://docs.rs/sansan)

</div>

---

## Documentation
1. For a user guide, see [`https://sansan.cat`](https://sansan.cat)
3. For a library reference, see [`https://docs.rs/sansan`](https://docs.rs/sansan)

## Applications
For real-world code usage examples of `sansan`, here's a table of projects that use `sansan` internally.

| Project | Example Code |
|---------|--------------|

## Design
`sansan`'s internal design is quite thoroughly documented, primarily to act as a reference to allow for easier changes in the future.

See [`DESIGN.md`](DESIGN.md) for the overview.

## Contributing
See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Audio Dependencies
This table summarizes the **audio-specific** libraries used by `sansan` and their purpose.

| Dependency              | Version  | Owner                                                       | Purpose |
|-------------------------|----------|-------------------------------------------------------------|---------|
| `audio_thread_priority` | `0.27.1` | [Mozilla](https://github.com/mozilla/audio_thread_priority) | Real-time audio thread promotion
| `cpal`                  | `0.15.2` | [RustAudio](https://github.com/RustAudio/cpal)              | Audio device input/output
| `souvlaki`              | `0.6.1`  | [Sinono3](https://github.com/Sinono3/souvlaki)              | OS media control interface
| `symphonia`             | `0.5.3`  | [Pdeljanov](https://github.com/pdeljanov/Symphonia)         | Audio demuxing/decoding/metadata
| `rubato`                | `0.14.1` | [HEnquist](https://github.com/HEnquist/rubato)              | Audio resampling

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
