	#[cfg(feature = "bulk")] #[cfg_attr(docsrs, doc(cfg(feature = "bulk")))]
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
				let mut probe = Self::new();
				chunk.iter().map(move |path| (path, probe.probe_path(path)))
			}).collect()
	}
