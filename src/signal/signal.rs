//---------------------------------------------------------------------------------------------------- use
use crate::state::{AudioState,ValidTrackData};
use crate::signal::{
	Add,Append,Back,Clear,Previous,RemoveRange,Remove,
	Repeat,Seek,SetIndex,Shuffle,Skip,Volume,
	AddError,SeekError,Next,NextError,PreviousError,SkipError,
	BackError,SetIndexError,RemoveError,RemoveRangeError,
};
use strum::{
	AsRefStr,Display,EnumCount,EnumVariantNames,
	EnumDiscriminants,IntoStaticStr,
};

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
	Previous(Previous),
	RemoveRange(RemoveRange),
	Remove(Remove),
	Repeat(Repeat),
	Seek(Seek),
	SetIndex(SetIndex),
	Shuffle(Shuffle),
	Skip(Skip),
	Volume(Volume),
	Next(Next),
}

//---------------------------------------------------------------------------------------------------- Signal Impl
impl Signal {
	fn add<T: ValidTrackData>(s: &mut Add, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn append<T: ValidTrackData>(s: &mut Append, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn back<T: ValidTrackData>(s: &mut Back, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn clear<T: ValidTrackData>(s: &mut Clear, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn previous<T: ValidTrackData>(s: &mut Previous, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn remove_range<T: ValidTrackData>(s: &mut RemoveRange, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn remove<T: ValidTrackData>(s: &mut Remove, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn repeat<T: ValidTrackData>(s: &mut Repeat, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn seek<T: ValidTrackData>(s: &mut Seek, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn set_index<T: ValidTrackData>(s: &mut SetIndex, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn shuffle<T: ValidTrackData>(s: &mut Shuffle, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn skip<T: ValidTrackData>(s: &mut Skip, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn volume<T: ValidTrackData>(s: &mut Volume, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}

	fn next<T: ValidTrackData>(s: &mut Next, w: &mut AudioState<T>, r: &AudioState<T>) {
		todo!();
	}
}

//---------------------------------------------------------------------------------------------------- someday::Apply
// TODO: just for trait bounds
impl<TrackData> someday::Apply<Signal> for AudioState<TrackData>
where
	TrackData: ValidTrackData,
{
	fn apply(patch: &mut Signal, writer: &mut Self, reader: &Self) {
		match patch {
			Signal::Add(signal)         => Signal::add(signal, writer, reader),
			Signal::Append(signal)      => Signal::append(signal, writer, reader),
			Signal::Back(signal)        => Signal::back(signal, writer, reader),
			Signal::Clear(signal)       => Signal::clear(signal, writer, reader),
			Signal::Previous(signal)    => Signal::previous(signal, writer, reader),
			Signal::RemoveRange(signal) => Signal::remove_range(signal, writer, reader),
			Signal::Remove(signal)      => Signal::remove(signal, writer, reader),
			Signal::Repeat(signal)      => Signal::repeat(signal, writer, reader),
			Signal::Seek(signal)        => Signal::seek(signal, writer, reader),
			Signal::SetIndex(signal)    => Signal::set_index(signal, writer, reader),
			Signal::Shuffle(signal)     => Signal::shuffle(signal, writer, reader),
			Signal::Skip(signal)        => Signal::skip(signal, writer, reader),
			Signal::Volume(signal)      => Signal::volume(signal, writer, reader),
			Signal::Next(signal)        => Signal::next(signal, writer, reader),
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
	Previous,
	RemoveRange,
	Remove,
	Repeat,
	Seek,
	SetIndex,
	Shuffle,
	Skip,
	Volume,
	Next,
}