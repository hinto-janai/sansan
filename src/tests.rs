//! These are helper functions used for testing throughout the codebase.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	engine::Engine,
	config::Config,
	source::Source,
};

//---------------------------------------------------------------------------------------------------- Test Init Helpers
/// Create a `Source` with a specified `usize` as the `Data`.
pub(crate) fn source(data: usize) -> Source<usize> {
	let path = std::path::Path::new("assets/audio/dialog-information.oga");
	Source::from((path, data))
}

// Init the `Engine` with default values and return
// 10 `Source`'s with `0..=9` as the `Data`.
pub(crate) fn init_test() -> (
	Engine::<usize, (), ()>,
	Vec<Source<usize>>,
) {
	let engine = Engine::<usize, (), ()>::init(Config::DEFAULT).unwrap();
	let vec = vec![
		source(0), source(1),
		source(2), source(3),
		source(4), source(5),
		source(6), source(7),
		source(8), source(9),
	];
	(engine, vec)
}

// Return 3 `Source`'s with `10, 20, 30` as the `Data`.
pub(crate) fn sources_10_20_30() -> Vec<Source<usize>> {
	vec![source(10), source(20), source(30)]
}

// Return 3 `Source`'s with `40, 50, 60` as the `Data`.
pub(crate) fn sources_40_50_60() -> Vec<Source<usize>> {
	vec![source(40), source(50), source(60)]
}

// Return 3 `Source`'s with `70, 80, 90` as the `Data`.
pub(crate) fn sources_70_80_90() -> Vec<Source<usize>> {
	vec![source(70), source(80), source(90)]
}