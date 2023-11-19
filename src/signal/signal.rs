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

// TODO: just for trait bounds
impl<TrackData> someday::Apply<Signal> for AudioState<TrackData>
where
	TrackData: ValidTrackData,
{
	fn apply(patch: &mut Signal, writer: &mut Self, reader: &Self) {
		todo!();
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