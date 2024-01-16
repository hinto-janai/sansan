//! TODO

//---------------------------------------------------------------------------------------------------- Use
use crate::{
	source::{Source,SourceInner},
	error::SourceError,
	valid_data::ExtraData,
};
use std::{
	time::Duration,
	io::Cursor,
	fs::File,
	path::{Path,PathBuf},
	sync::Arc,
	borrow::Cow,
};
use symphonia::core::{
	formats::{FormatReader,FormatOptions},
	io::{MediaSourceStream, MediaSourceStreamOptions},
	probe::Hint,
	meta::{MetadataOptions,Limit},
	units::{Time,TimeBase},
	codecs::{Decoder, DecoderOptions},
};
use symphonia::default::{get_probe,get_codecs};

#[allow(unused_imports)] // docs
use crate::state::AudioState;

//---------------------------------------------------------------------------------------------------- Constants
// `symphonia` format options.
//
// These are some misc options `Symphonia` needs.
// Most of these are the default values, but as `const`.

/// TODO
pub(crate) const FORMAT_OPTIONS: FormatOptions = FormatOptions {
	enable_gapless: true,
	prebuild_seek_index: false,
	seek_index_fill_rate: 20,
};

/// TODO
pub(crate) const METADATA_OPTIONS: MetadataOptions = MetadataOptions {
	limit_metadata_bytes: Limit::Default,
	limit_visual_bytes: Limit::Default,
};

/// TODO
pub(crate) const DECODER_OPTIONS: DecoderOptions = DecoderOptions {
	verify: false,
};

/// TODO
pub(crate) const MEDIA_SOURCE_STREAM_OPTIONS: MediaSourceStreamOptions = MediaSourceStreamOptions {
	buffer_len: 64 * 1024,
};

//---------------------------------------------------------------------------------------------------- SourceDecode
/// The type the `Decoder` thread wants.
///
/// This is the type `Decoder` holds onto when decoding a track.
/// It contains the necessary data to decode a particular track,
/// and is created from the public API `Source` type.
pub(crate) struct SourceDecode {
	/// The current audio file/sound/source.
	pub(crate) reader: Box<dyn FormatReader>,
	/// The current audio's decoder
	pub(crate) decoder: Box<dyn Decoder>,
	/// The audio's sample rate
	pub(crate) sample_rate: u32,
	/// The audio's current `Time`
	pub(crate) time_now: Time,
	/// The audio's total runtime.
	/// This is calculated in `try_from_inner()` before any decoding.
	pub(crate) time_total: Time,
	/// Same as above, but in [f64] seconds.
	pub(crate) secs_total: f64,
	/// The audio's `TimeBase`.
	/// This is used to calculated elapsed time as the audio progresses.
	pub(crate) timebase: TimeBase,
}

impl SourceDecode {
	#[cold]
	#[inline(never)]
	/// Returns a dummy [`SourceDecode`]
	/// that cannot actually be used.
	///
	/// This exists so [Decode] does not
	/// have to keep an [Option<SourceDecode>].
	///
	/// INVARIANT:
	/// This must not actually be _used_, as in the
	/// trait functions must not be called as they
	/// all panic.
	pub(crate) fn dummy() -> Self {
		use symphonia::core::{
			errors::Result,
			formats::{Cue,SeekMode,SeekTo,SeekedTo,Track,Packet},
			meta::Metadata,
			codecs::{CodecParameters,CodecDescriptor,FinalizeResult},
			audio::AudioBufferRef,
		};

		/// TODO
		struct DummyReader;
		#[allow(clippy::panic_in_result_fn)]
		impl FormatReader for DummyReader {
			#[cold] #[inline(never)]
			fn try_new(source: MediaSourceStream, options: &FormatOptions) -> Result<Self> { unreachable!() }
			#[cold] #[inline(never)]
			fn cues(&self) -> &[Cue] { unreachable!() }
			#[cold] #[inline(never)]
			fn metadata(&mut self) -> Metadata<'_> { unreachable!() }
			#[cold] #[inline(never)]
			fn seek(&mut self, mode: SeekMode, to: SeekTo) -> Result<SeekedTo> { unreachable!() }
			#[cold] #[inline(never)]
			fn tracks(&self) -> &[Track] { unreachable!() }
			#[cold] #[inline(never)]
			fn next_packet(&mut self) -> Result<Packet> { unreachable!() }
			#[cold] #[inline(never)]
			fn into_inner(self: Box<Self>) -> MediaSourceStream { unreachable!() }
		}

		/// TODO
		struct DummyDecoder;
		#[allow(clippy::panic_in_result_fn)]
		impl Decoder for DummyDecoder {
			#[cold] #[inline(never)]
			fn try_new(params: &symphonia::core::codecs::CodecParameters, options: &DecoderOptions) -> Result<Self> { unreachable!() }
			#[cold] #[inline(never)]
			fn supported_codecs() -> &'static [CodecDescriptor] { unreachable!() }
			#[cold] #[inline(never)]
			fn reset(&mut self) { unreachable!() }
			#[cold] #[inline(never)]
			fn codec_params(&self) -> &CodecParameters { unreachable!() }
			#[cold] #[inline(never)]
			fn decode(&mut self, packet: &Packet) -> Result<AudioBufferRef> { unreachable!() }
			#[cold] #[inline(never)]
			fn finalize(&mut self) -> FinalizeResult { unreachable!() }
			#[cold] #[inline(never)]
			fn last_decoded(&self) -> AudioBufferRef { unreachable!() }
		}

		Self {
			reader:      Box::new(DummyReader),
			decoder:     Box::new(DummyDecoder),
			sample_rate: 0,
			time_now:    Time { seconds: 0, frac: 0.0, },
			time_total:  Time { seconds: 0, frac: 0.0 },
			secs_total:  0.0,
			timebase:    TimeBase { numer: 0, denom: 0 },
		}
	}
}

//---------------------------------------------------------------------------------------------------- MediaSourceStream -> SourceDecode
impl TryFrom<MediaSourceStream> for SourceDecode {
	type Error = SourceError;

	fn try_from(mss: MediaSourceStream) -> Result<Self, Self::Error> {
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
			return Err(SourceError::Current);
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
		let time_total = timebase.calc_time(n_frames);
		let secs_total = time_total.seconds as f64 + time_total.frac;

		Ok(Self {
			reader,
			decoder,
			sample_rate,
			time_now: Time { seconds: 0, frac: 0.0 },
			time_total,
			secs_total,
			timebase,
		})
	}
}

//---------------------------------------------------------------------------------------------------- Source -> SourceDecode
impl<Data> TryInto<SourceDecode> for Source<Data>
where
	Data: ExtraData
{
	type Error = SourceError;

	fn try_into(self) -> Result<SourceDecode, Self::Error> {
		match self.0 {
			SourceInner::ArcPath(path) => {
				let file = File::open(path.0)?;
				let mss = MediaSourceStream::new(
					Box::new(file),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
			SourceInner::CowPath(path) => {
				let file = File::open(path.0)?;
				let mss = MediaSourceStream::new(
					Box::new(file),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
			SourceInner::ArcByte(bytes) => {
				let cursor = Cursor::new(bytes.0);
				let mss = MediaSourceStream::new(
					Box::new(cursor),
					MEDIA_SOURCE_STREAM_OPTIONS,
				);
				mss.try_into()
			},
			SourceInner::CowByte(bytes) => {
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