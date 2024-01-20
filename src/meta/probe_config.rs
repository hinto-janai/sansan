//! TODO

//---------------------------------------------------------------------------------------------------- Use
use std::{
	sync::Arc,
	collections::HashMap,
};

//---------------------------------------------------------------------------------------------------- ProbeConfig
/// TODO
#[allow(clippy::struct_excessive_bools)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Copy,Clone,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub struct ProbeConfig {
	/// TODO
	pub artist_name: bool,
	/// TODO
	pub album_title: bool,
	/// TODO
	pub track_title: bool,
	/// TODO
	pub total_runtime: bool,
	/// TODO
	pub sample_rate: bool,
	/// TODO
	pub track_number: bool,
	/// TODO
	pub disc_number: bool,
	/// TODO
	pub cover_art: bool,
	/// TODO
	pub release_date: bool,
	/// TODO
	pub genre: bool,
	/// TODO
	pub compilation: bool,
}

impl ProbeConfig {
	/// All set to `true` except [`Self::cover_art`].
	pub const NO_COVER_ART: Self = Self {
		artist_name:   true,
		album_title:   true,
		track_title:   true,
		total_runtime: true,
		sample_rate:   true,
		track_number:  true,
		disc_number:   true,
		cover_art:     false,
		release_date:  true,
		genre:         true,
		compilation:   true,
	};

	/// All set to `true`.
	pub const ALL: Self = Self {
		artist_name:   true,
		album_title:   true,
		track_title:   true,
		total_runtime: true,
		sample_rate:   true,
		track_number:  true,
		disc_number:   true,
		cover_art:     true,
		release_date:  true,
		genre:         true,
		compilation:   true,
	};

	/// All set to `false`.
	pub const NONE: Self = Self {
		artist_name:   false,
		album_title:   false,
		track_title:   false,
		total_runtime: false,
		sample_rate:   false,
		track_number:  false,
		disc_number:   false,
		cover_art:     false,
		release_date:  false,
		genre:         false,
		compilation:   false,
	};

	#[must_use]
	/// If all values are set to `true`.
	pub const fn all(&self) -> bool {
		self.artist_name   &&
		self.album_title   &&
		self.track_title   &&
		self.total_runtime &&
		self.sample_rate   &&
		self.track_number  &&
		self.disc_number   &&
		self.cover_art     &&
		self.release_date  &&
		self.genre         &&
		self.compilation
	}

	#[must_use]
	/// If all values are set to `false`.
	pub const fn none(&self) -> bool {
		!self.artist_name   &&
		!self.album_title   &&
		!self.track_title   &&
		!self.total_runtime &&
		!self.sample_rate   &&
		!self.track_number  &&
		!self.disc_number   &&
		!self.cover_art     &&
		!self.release_date  &&
		!self.genre         &&
		!self.compilation
	}
}

impl Default for ProbeConfig {
	///  [`Self::ALL`].
	fn default() -> Self {
		Self::ALL
	}
}