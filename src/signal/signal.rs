//---------------------------------------------------------------------------------------------------- use
use crate::state::{AudioState,ValidTrackData};
use crate::signal::{
	Add,AddMany,Back,Clear,Previous,RemoveRange,Remove,
	Repeat,Seek,SetIndex,Shuffle,Skip,Volume,Play,Pause,
	Toggle,Stop,InsertMethod,
	AddError,SeekError,Next,NextError,PreviousError,SkipError,
	BackError,SetIndexError,RemoveError,RemoveRangeError,
	AddManyError,
};
use strum::{
	AsRefStr,Display,EnumCount,EnumVariantNames,
	EnumDiscriminants,IntoStaticStr,
};
use someday::{Apply,ApplyReturn};

//---------------------------------------------------------------------------------------------------- AudioState Apply (someday)
#[derive(Clone,Debug,PartialEq,PartialOrd)]
#[derive(AsRefStr,Display,EnumCount,EnumVariantNames,EnumDiscriminants,IntoStaticStr)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// #[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Signal<TrackData: ValidTrackData> {
	Add(Add<TrackData>),
	AddMany(AddMany<TrackData>),
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
	Stop(Stop),
	Toggle(Toggle),
	Volume(Volume),
}

//---------------------------------------------------------------------------------------------------- someday::Apply
// TODO: just for trait bounds
macro_rules! todo_impl_signal {
	($($signal:ident),* $(,)?) => {
		$(
			impl<TrackData: ValidTrackData> ApplyReturn<Signal<TrackData>, $signal, ()> for AudioState<TrackData> {
				fn apply_return(_: &mut $signal, _: &mut Self, _: &Self) {
					todo!();
				}
			}
		)*
	}
}
todo_impl_signal!(Back,Previous,RemoveRange,Remove,Repeat,Seek,SetIndex,Skip,Next);

// [Apply] will just call the [ApplyReturn::apply_return]
// implementation found in each respective signal's file.
impl<TrackData: ValidTrackData> Apply<Signal<TrackData>> for AudioState<TrackData> {
	fn apply(patch: &mut Signal<TrackData>, writer: &mut Self, reader: &Self) {
		match patch {
			Signal::Add(signal)         => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::AddMany(signal)     => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Back(signal)        => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Clear(signal)       => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Play(signal)        => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Pause(signal)       => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Previous(signal)    => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::RemoveRange(signal) => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Remove(signal)      => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Repeat(signal)      => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Seek(signal)        => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::SetIndex(signal)    => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Skip(signal)        => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Stop(signal)        => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Toggle(signal)      => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Volume(signal)      => drop(ApplyReturn::apply_return(signal, writer, reader)),
			Signal::Next(signal)        => drop(ApplyReturn::apply_return(signal, writer, reader)),

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
	($($signal:ident),* $(,)?) => {
		$(
			impl<TrackData: ValidTrackData> From<$signal> for Signal<TrackData> {
				fn from(value: $signal) -> Self {
					Signal::$signal(value)
				}
			}
		)*
	}
}
impl_from!(Back,Clear,Next,Play,Pause,Previous,RemoveRange,Remove,Repeat,Seek,SetIndex,Shuffle,Skip,Stop,Toggle,Volume);

macro_rules! impl_from_generic {
	($($signal:ident),* $(,)?) => {
		$(
			impl<TrackData: ValidTrackData> From<$signal<TrackData>> for Signal<TrackData> {
				fn from(value: $signal<TrackData>) -> Self {
					Signal::$signal(value)
				}
			}
		)*
	}
}
impl_from_generic!(Add,AddMany);