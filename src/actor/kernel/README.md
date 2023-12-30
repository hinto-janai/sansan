# Kernel
`Kernel` is in its own module due to the vast separation of all signal handlers that it has to handle, e.g:
- `play()`
- `seek()`
- `set_index()`

Instead of defining them all in `kernel.rs` one after the other, [creating 1 large un-readable, un-maintainable file](https://github.com/hinto-janai/festival/blob/e1d479ca40efbda683b3e2c7d81ffb6e59391698/shukusai/src/audio/audio.rs#L727-L1369), each signal is separated into its own file proper.

Each signal can have extensive tests and other helper functions all without blowing the 1 file up.

TODO: add `Engine` <-> `Kernel` <-> `Signal` tests

This doesn't change functionality, although, maintaining `10 * 300 line` files is a much better feeling than `1 * 3000 line` file.

These are all defined as a `kernel.rs/Kernel` method, and in the same shape, i.e:
```rust
//---------------------------------------------- Signal Implementation
impl<Data: ValidData> Kernel<Data> {
	fn signal(&mut self) {
		/* ... */
	}
}

//---------------------------------------------- Tests
#[cfg(test)]
mod tests {
	fn test_for_signal() {
		/* ... */
	}
}
```