//---------------------------------------------------------------------------------------------------- use
use crate::api::engine::Engine;

//---------------------------------------------------------------------------------------------------- Config
#[cfg_attr(feature = "serde", serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "bincode", bincode::Encode, bincode::Decode)]
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
pub struct Callbacks<F, I, T>
where
	F: FnMut(&mut I, &mut Engine<T>),
	T: Clone,
{
	input: I,
	next: Option<F>,
	elapsed: Option<(F, std::time::Duration)>,
	queue_end: Option<F>,
	repeat: Option<F>,

	_t: std::marker::PhantomData<T>,
}

//---------------------------------------------------------------------------------------------------- Config