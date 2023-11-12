//---------------------------------------------------------------------------------------------------- Use
use std::{
	io::{Read,Seek,Cursor},
	fs::File,
	path::{Path,PathBuf},
	sync::Arc,
	rc::Rc,
};
use symphonia::core::{
	formats::{FormatReader,FormatOptions},
	io::{MediaSourceStream, MediaSourceStreamOptions},
	probe::{Probe,Hint},
	meta::{MetadataOptions,Limit},
	units::{Time,TimeBase},
	codecs::{Decoder, DecoderOptions},
};
use symphonia::default::{
	get_probe,get_codecs,
};
use crate::{source::Source, MediaControlMetadata};

//---------------------------------------------------------------------------------------------------- Constants
// `symphonia` format options.
//
// These are some misc options `Symphonia` needs.
// Most of these are the default values, but as `const`.

const FORMAT_OPTIONS: FormatOptions = FormatOptions {
	enable_gapless: true,
	prebuild_seek_index: false,
	seek_index_fill_rate: 20,
};

const METADATA_OPTIONS: MetadataOptions = MetadataOptions {
	limit_metadata_bytes: Limit::Default,
	limit_visual_bytes: Limit::Default,
};

const DECODER_OPTIONS: DecoderOptions = DecoderOptions {
	verify: false,
};

const MEDIA_SOURCE_STREAM_OPTIONS: MediaSourceStreamOptions = MediaSourceStreamOptions {
	buffer_len: 64 * 1024,
};

//---------------------------------------------------------------------------------------------------- SourceInner
// The type the `Decoder` thread wants.
//
// This is the type `Decoder` holds onto when decoding a track.
// It contains the necessary data to decode a particular track,
// and is created from the public API `Source` type.
pub(super) struct SourceInner {
	// The current audio file/sound/source.
	pub(super) reader: Box<dyn FormatReader>,
	// The current audio's decoder
	pub(super) decoder: Box<dyn Decoder>,
	// The audio's sample rate
	pub(super) sample_rate: u32,
	// The audio's current `Time`
	pub(super) time: Time,
	// The audio's `TimeBase`.
	// This is used to calculated elapsed time as the audio progresses.
	pub(super) timebase: TimeBase,
	// The audio's total runtime.
	// This is calculated in `try_from_inner()` before any decoding.
	pub(super) total_time: Time,
}

//---------------------------------------------------------------------------------------------------- MediaSourceStream -> SourceInner
impl TryFrom<MediaSourceStream> for SourceInner {
	type Error = SourceError;

	fn try_from(mss: MediaSourceStream) -> Result<SourceInner, Self::Error> {
		let result = get_probe().format(
			&Hint::new(),
			mss,
			&FORMAT_OPTIONS,
			&METADATA_OPTIONS
		)?;

		let reader = result.format;

		// TODO:
		// These lazy's should be initialized early on in the `Engine` init phase.
		let codecs = symphonia::default::get_codecs();

		// Select the first track with a known codec.
		let Some(track) = reader
			.tracks()
			.iter()
			.find(|t| {
				// Make sure it is not null.
				t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL &&
				// And it exists in our codec registry.
				codecs.get_codec(t.codec_params.codec).is_some()
			})
		else {
			return Err(SourceError::Track);
		};

		// Create a decoder for the track.
		let decoder = match get_codecs().make(&track.codec_params, &DECODER_OPTIONS) {
			Ok(d) => d,
			Err(e) => return Err(SourceError::Decoder(e.into())),
		};

		// Get sample rate.
		let Some(sample_rate) = track.codec_params.sample_rate else {
			return Err(SourceError::SampleRate);
		};

		// Get timebase.
		let Some(timebase) = track.codec_params.time_base else {
			return Err(SourceError::TimeBase);
		};

		// Calculate total runtime of audio.
		let Some(n_frames) = track.codec_params.n_frames else {
			return Err(SourceError::Frames);
		};
		let total_time = timebase.calc_time(n_frames);

		Ok(Self {
			reader,
			decoder,
			sample_rate,
			time: Time { seconds: 0, frac: 0.0 },
			timebase,
			total_time,
		})
	}
}

//---------------------------------------------------------------------------------------------------- Source -> SourceInner
impl TryInto<SourceInner> for Source {
	type Error = SourceError;

	#[inline]
	fn try_into(self) -> Result<SourceInner, Self::Error> {
		match self {
			Self::Path(path) => {
				let file = File::open(path.0)?;
				let mss = MediaSourceStream::new(
					Box::new(file),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
			Self::Bytes(bytes) => {
				let cursor = Cursor::new(bytes.0);
				let mss = MediaSourceStream::new(
					Box::new(cursor),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
		}
	}
}

//---------------------------------------------------------------------------------------------------- Source Errors
#[derive(thiserror::Error, Debug)]
/// Errors when loading a [`Source`]
///
/// This `enum` represents all the potential errors that can
/// occur when attempting to load a [`Source`] into a viable
/// audio container.
///
/// This includes things like:
/// - The data not actually be audio
/// - File IO errors (non-existent PATH, lacking-permissions, etc)
/// - Unsupported audio codec
pub enum SourceError {
	#[error("failed to open file: {0}")]
	/// Error occurred while reading a [`File`] (most likely missing)
	File(#[from] std::io::Error),

	#[error("failed to probe audio data: {0}")]
	/// Error occurred while attempting to probe the audio data
	Probe(#[from] symphonia::core::errors::Error),

	#[error("failed to create codec decoder: {0}")]
	/// Error occurred while creating a decoder for the audio codec
	Decoder(#[from] DecoderError),

	#[error("failed to find track within the codec")]
	/// The audio codec did not specify a track
	Track,

	#[error("failed to find the codecs sample rate")]
	/// The audio codec did not specify a sample rate
	SampleRate,

	#[error("failed to find codec time")]
	/// The audio codec did not include a timebase
	TimeBase,

	#[error("failed to find codec n_frames")]
    /// The audio codec did not specify the number of frames
	Frames,
}

#[derive(thiserror::Error, Debug)]
pub enum DecoderError {
	#[error("the audio data contained malformed data")]
	/// The audio data contained malformed data
    Decode(&'static str),

	#[error("codec/container is not supported")]
	/// Codec/container is not supported
    Unsupported(&'static str),

	#[error("a limit was reached while decoding")]
	/// A limit was reached while decoding
    Limit(&'static str),

	#[error("decoding io error")]
	/// Unknown IO error
    Io(#[from] std::io::Error),

	#[error("unknown decoding error")]
	/// Unknown decoding error
	Unknown,
}

impl From<symphonia::core::errors::Error> for DecoderError {
	fn from(value: symphonia::core::errors::Error) -> Self {
		use symphonia::core::errors::Error as E;
		match value {
			E::DecodeError(s) => Self::Decode(s),
			E::Unsupported(s) => Self::Unsupported(s),
			E::LimitError(s)  => Self::Limit(s),
			E::IoError(s)     => Self::Io(s),
			_ => Self::Unknown,
		}
	}
}