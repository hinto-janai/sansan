//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::time::Duration;

//---------------------------------------------------------------------------------------------------- EngineInitError
#[derive(thiserror::Error)]
#[derive(Debug)]
///
pub enum EngineInitError {
	#[error("failed to spawn thread `{name}`: {error}")]
	/// Failed to spawn an OS thread
	ThreadSpawn {
		/// Name of the thread that failed to spawn
		name: &'static str,
		/// Associated IO error
		error: std::io::Error,
	}
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod tests {}