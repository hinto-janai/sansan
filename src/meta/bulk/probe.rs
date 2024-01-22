//! Batch probing.

//---------------------------------------------------------------------------------------------------- Use
use crate::meta::{
	constants::INFER_AUDIO_PREFIX_LEN,
	Metadata,
	Probe,
	ProbeError,
};
use std::{
	fs::File,
	path::Path,
	borrow::Cow,
	time::Duration,
	io::Cursor,
};
use symphonia::core::{
	formats::Track,
	meta::{Tag,StandardTagKey,Visual},
};
use std::sync::OnceLock;

//---------------------------------------------------------------------------------------------------- Probe
/// TODO
pub fn probe_path_bulk<P>(paths: &[P]) -> Vec<(&P, Result<Metadata, ProbeError>)>
where
	P: AsRef<Path> + Sync,
{
	use rayon::prelude::*;

	// Only use 25% of threads.
	// More threads start to impact negatively due
	// to this mostly being a heavy I/O operation.
	let threads = crate::free::threads().get() / 4;
	let chunk_size = paths.len() / threads;

	paths
		.par_chunks(chunk_size)
		.flat_map_iter(|chunk| {
			let mut probe = Probe::new();
			chunk.iter().map(move |path| (path, probe.probe_path(path)))
		}).collect()
}
