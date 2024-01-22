//! Audio metadata.

#[cfg(feature = "bulk")]
#[cfg_attr(docsrs, doc(cfg(feature = "bulk")))]
pub mod bulk;

mod metadata;
pub use metadata::Metadata;

mod map;
pub use map::Map;

mod track;
pub use track::Track;

mod probe;
pub use probe::Probe;

mod probe_error;
pub use probe_error::ProbeError;

pub(crate) mod extract;

mod mime;
pub use mime::{AudioMime,AudioMimeProbe};

mod constants;
pub use constants::{
	SUPPORTED_AUDIO_MIME_TYPES,
	SUPPORTED_IMG_MIME_TYPES,
};

mod free;
pub use free::{
	probe_path,
	probe_file,
	probe_bytes,
	is_audio, is_audio_path,
	is_midi, is_midi_path,
	is_mp3, is_mp3_path,
	is_m4a, is_m4a_path,
	is_ogg_opus, is_ogg_opus_path,
	is_ogg, is_ogg_path,
	is_flac, is_flac_path,
	is_wav, is_wav_path,
	is_amr, is_amr_path,
	is_aac, is_aac_path,
	is_aiff, is_aiff_path,
	is_dsf, is_dsf_path,
	is_ape, is_ape_path,
};