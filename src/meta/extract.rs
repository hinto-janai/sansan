//! Free functions for extracting data from `symphonia` types.
//!
//! These are only used internally for `Probe`.

//---------------------------------------------------------------------------------------------------- Use
use std::{
	borrow::Cow,
	time::Duration,
};
use symphonia::core::{
	formats::Track,
	meta::{Tag,StandardTagKey,Visual},
};

//---------------------------------------------------------------------------------------------------- Extraction functions
/// Get `artist` tag.
pub(super) fn artist_name(tags: &[Tag]) -> Option<Cow<'_, str>> {
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::AlbumArtist)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	// This isn't first because many `Artist` metadata
	// fields contain the featured artists, e.g `Artist A x Artist B`.
	// `AlbumArtist` usually contains just the main `Artist` name, which we want.
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::Artist)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::Composer)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::Performer)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::OriginalArtist)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	None
}

/// Attempt to get album title.
pub(super) fn album_title(tags: &[Tag]) -> Option<Cow<'_, str>> {
	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::Album)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	if let Some(t) = tags.iter().find(|i| i.std_key == Some(StandardTagKey::OriginalAlbum)) {
		let o = value(t);
		if o.is_some() { return o; }
	}

	None
}

/// Attempt to get song title.
pub(super) fn track_title(tags: &[Tag]) -> Option<Cow<'_, str>> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::TrackTitle))
		.and_then(value)
}

/// Attempt to get track number.
pub(super) fn track_number(tags: &[Tag]) -> Option<u32> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::TrackNumber))
		.and_then(value_unsigned)
}

/// Get a tracks sample rate.
pub(super) const fn sample_rate(track: &Track) -> Option<u32> {
	track.codec_params.sample_rate
}

/// Get a tracks runtime.
pub(super) fn total_runtime(track: &Track) -> Option<Duration> {
	let Some(timestamp) = track.codec_params.n_frames else {
		return None;
	};

	let Some(time) = track.codec_params.time_base else {
		return None;
	};

	let time = time.calc_time(timestamp);
	let total = time.seconds as f64 + time.frac;

	Some(Duration::from_secs_f64(total))
}

/// Attempt to get track disc number.
pub(super) fn disc_number(tags: &[Tag]) -> Option<u32> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::DiscNumber))
		.and_then(value_unsigned)
}

/// Attempt to get the release date.
pub(super) fn release_date(tags: &[Tag]) -> Option<Cow<'_, str>> {
	tags
		.iter()
		.find(|i| {
			i.std_key == Some(StandardTagKey::Date) ||
			i.std_key == Some(StandardTagKey::ReleaseDate) ||
			i.std_key == Some(StandardTagKey::OriginalDate)
		})
		.and_then(value)
}

/// Attempt to get the genre.
pub(super) fn genre(tags: &[Tag]) -> Option<Cow<'_, str>> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::Genre))
		.and_then(value)
}

/// Attempt to get the art bytes.
pub(super) fn cover_art(visuals: &[Visual]) -> Option<&[u8]> {
	// Find the biggest visual and return it.
	visuals
		.iter()
		.max_by(|a, b| a.data.len().cmp(&b.data.len()))
		.map(|biggest| &*biggest.data)
}

/// Get the compilation bool.
/// Assume `false` if it doesn't exist.
pub(super) fn compilation(tags: &[Tag]) -> Option<bool> {
	tags
		.iter()
		.find(|i| i.std_key == Some(StandardTagKey::Compilation))
		.map(value_bool)
}

/// Extract a `Tag`'s `Value` to a string.
///
/// This expects values that are supposed to be strings.
///
/// If the value is empty, this returns none.
fn value(tag: &Tag) -> Option<Cow<'_, str>> {
	use symphonia::core::meta::Value;
	match &tag.value {
		Value::String(s) => s.split_whitespace().next().map(Cow::Borrowed),
		Value::Binary(b) => {
			if let Ok(s) = std::str::from_utf8(b) {
				s.split_whitespace().next().map(Cow::Borrowed)
			} else {
				None
			}
		},
		Value::UnsignedInt(u) => Some(Cow::Owned(u.to_string())),
		Value::SignedInt(s)   => Some(Cow::Owned(s.to_string())),
		Value::Float(f)       => Some(Cow::Owned(f.to_string())),
		Value::Boolean(b)     => Some(Cow::Borrowed(if *b { "true" } else { "false" })),
		Value::Flag           => None,
	}
}

/// Extract a `Tag`'s `Value` to a number.
///
/// This expects values that are supposed to be unsigned integers.
fn value_unsigned(tag: &Tag) -> Option<u32> {
	use symphonia::core::meta::Value;
	match &tag.value {
		Value::UnsignedInt(u) => Some(*u as u32),
		Value::SignedInt(s)   => Some(*s as u32),
		Value::Float(f)       => Some(*f as u32),
		Value::Boolean(b) => {
			match b {
				true  => Some(1),
				false => Some(0),
			}
		},
		Value::String(s) => {
			if let Ok(u) = s.parse::<u32>() {
				Some(u)
			// Some `TrackNumber` fields are strings like `1/12`.
			} else if let Some(u) = s.split('/').next() {
				u.parse::<u32>().ok()
			} else {
				None
			}
		},
		Value::Binary(b) => {
			match std::str::from_utf8(b) {
				Ok(s) => {
					if let Ok(u) = s.parse::<u32>() {
						Some(u)
					} else if let Some(u) = s.split('/').next() {
						u.parse::<u32>().ok()
					} else {
						None
					}
				},
				_ => None,
			}
		},
		Value::Flag => None,
	}
}

/// Extract a `Tag`'s `Value` to a bool
///
/// This expects values that are supposed to be bool.
fn value_bool(tag: &Tag) -> bool {
	use symphonia::core::meta::Value;
	match &tag.value {
		Value::Boolean(b) => *b,
		Value::String(s) => {
			match s.parse::<bool>() {
				Ok(b) => b,
				_     => false,
			}
		},
		Value::Binary(b) => {
			match std::str::from_utf8(b) {
				Ok(s) => {
					match s.parse::<bool>() {
						Ok(b) => b,
						_     => false,
					}
				},
				_ => false,
			}
		},

		Value::Flag => true,
		Value::Float(f) => f != &0.0,
		Value::SignedInt(i) => i != &0,
		Value::UnsignedInt(u) => u != &0,
	}
}