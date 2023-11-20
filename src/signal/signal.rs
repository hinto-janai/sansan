//---------------------------------------------------------------------------------------------------- use
use crate::state::{AudioState,ValidTrackData};
use crate::signal::{
	Add,Append,Back,Clear,Previous,RemoveRange,Remove,
	Repeat,Seek,SetIndex,Shuffle,Skip,Volume,Play,Pause,Toggle,
	AddError,SeekError,Next,NextError,PreviousError,SkipError,
	BackError,SetIndexError,RemoveError,RemoveRangeError,
};
use strum::{
	AsRefStr,Display,EnumCount,EnumVariantNames,
	EnumDiscriminants,IntoStaticStr,
};
use someday::{Apply,ApplyReturn};

//---------------------------------------------------------------------------------------------------- AudioState Apply (someday)
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
#[derive(AsRefStr,Display,EnumCount,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub(crate) enum Signal {
	Add(Add),
	Append(Append),
	Back(Back),
	Clear(Clear),
	Next(Next),
	Play(Play),
	Pause(Pause),
	Previous(Previous),
	RemoveRange(RemoveRange),
	Remove(Remove),
	Repeat(Repeat),
	Seek(Seek),
	SetIndex(SetIndex),
	Shuffle(Shuffle),
	Skip(Skip),
	Toggle(Toggle),
	Volume(Volume),
}

//---------------------------------------------------------------------------------------------------- someday::Apply
// TODO: just for trait bounds
macro_rules! todo_impl_signal {
	($($signal:ident),* $(,)?) => {$(
		impl<TrackData: ValidTrackData> ApplyReturn<Signal, $signal, ()> for AudioState<TrackData> {
			fn apply_return(s: &mut $signal, w: &mut Self, r: &Self) {
				todo!();
			}
		}
	)*};
}
todo_impl_signal!(Add,Append,Back,Clear,Previous,RemoveRange,Remove,Repeat,Seek,SetIndex,Skip,Next);

// [Apply] will just call the [ApplyReturn::apply_return]
// implementation found in each respective signal's file.
impl<TrackData: ValidTrackData> Apply<Signal> for AudioState<TrackData> {
	fn apply(patch: &mut Signal, writer: &mut Self, reader: &Self) {
		match patch {
			Signal::Add(signal)         => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Append(signal)      => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Back(signal)        => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Clear(signal)       => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Play(signal)        => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Pause(signal)       => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Previous(signal)    => ApplyReturn::apply_return(signal, writer, reader),
			Signal::RemoveRange(signal) => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Remove(signal)      => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Repeat(signal)      => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Seek(signal)        => ApplyReturn::apply_return(signal, writer, reader),
			Signal::SetIndex(signal)    => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Skip(signal)        => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Toggle(signal)      => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Volume(signal)      => ApplyReturn::apply_return(signal, writer, reader),
			Signal::Next(signal)        => ApplyReturn::apply_return(signal, writer, reader),

			// SAFETY:
			// Patches must be deterministic across writer/reader [Apply]'s,
			// however, [Shuffle] introduces RNG on the writer side which cannot easily
			// be replicated from the readers, so, when [Kernel] calls [shuffle()],
			// it will use [writer.push_clone()] such that readers will always clone
			// data instead, meaning this branch will (should) never be taken.
			//
			// [writer.sync()] should also never be taken
			// as that gets skipped if [push_clone()] is used.
			Signal::Shuffle(_) => crate::macros::unreachable2!(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- Impl From
macro_rules! impl_from {
	($($signal:ident),* $(,)?) => { $(
		impl From<$signal> for Signal {
			fn from(value: $signal) -> Self {
				Signal::$signal(value)
			}
		}
	)* }
}

impl_from! {
	Add,
	Append,
	Back,
	Clear,
	Next,
	Play,
	Pause,
	Previous,
	RemoveRange,
	Remove,
	Repeat,
	Seek,
	SetIndex,
	Shuffle,
	Skip,
	Toggle,
	Volume,
}