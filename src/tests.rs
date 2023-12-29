//! These are helper functions used for testing throughout the codebase.

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	engine::Engine,
	config::Config,
	source::{Source,Sources},
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
	Sources<usize>,
) {
	let engine = Engine::<usize, (), ()>::init(Config::DEFAULT).unwrap();
	let sources = Sources::from_10([
		source(0), source(1),
		source(2), source(3),
		source(4), source(5),
		source(6), source(7),
		source(8), source(9),
	]);
	(engine, sources)
}

// Return 3 `Source`'s with `10, 20, 30` as the `Data`.
pub(crate) fn sources_10_20_30() -> Sources<usize> {
	Sources::from_3([source(10), source(20), source(30)])
}

// Return 3 `Source`'s with `11, 22, 33` as the `Data`.
pub(crate) fn sources_11_22_33() -> Sources<usize> {
	Sources::from_3([source(11), source(22), source(33)])
}

// Return 3 `Source`'s with `40, 50, 60` as the `Data`.
pub(crate) fn sources_40_50_60() -> Sources<usize> {
	Sources::from_3([source(40), source(50), source(60)])
}

// Return 3 `Source`'s with `44, 55, 66` as the `Data`.
pub(crate) fn sources_44_55_66() -> Sources<usize> {
	Sources::from_3([source(44), source(55), source(66)])
}

// Return 3 `Source`'s with `70, 80, 90` as the `Data`.
pub(crate) fn sources_70_80_90() -> Sources<usize> {
	Sources::from_3([source(70), source(80), source(90)])
}

// Return 3 `Source`'s with `77, 88, 99` as the `Data`.
pub(crate) fn sources_77_88_99() -> Sources<usize> {
	Sources::from_3([source(77), source(88), source(99)])
}