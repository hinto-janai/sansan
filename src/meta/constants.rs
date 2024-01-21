//! Metadata constants.

/// The capacity of the buffer used for `infer` usage.
///
/// We only need the first 36 bytes from a
/// file to determine if it is audio or not.
/// <https://docs.rs/infer/0.15.0/src/infer/matchers/audio.rs.html#38>
///
/// `infer::get_from_path` creates and uses an internal buffer
/// each time that potentially allocates/reads 8192 bytes
/// (<https://docs.rs/infer/0.15.0/src/infer/lib.rs.html#248>),
/// so instead, each `Probe` will come with a re-usable buffer
/// with this specified capacity.
pub(super) const INFER_AUDIO_PREFIX_LEN: usize = 36;

/// TODO
///
/// Sorted for binary search.
pub const SUPPORTED_AUDIO_MIME_TYPES: [&str; 26] = [
	"audio/aac",
	"audio/adpcm",
	"audio/flac",
	"audio/m4a",
	"audio/mp3",
	"audio/mpeg",
	"audio/mpeg3",
	"audio/ogg",
	"audio/opus",
	"audio/vorbis",
	"audio/wav",
	"audio/wavpack",
	"audio/wavpack-correction",
	"audio/x-aac",
	"audio/x-adpcm",
	"audio/x-flac",
	"audio/x-m4a",
	"audio/x-mp3",
	"audio/x-mpeg",
	"audio/x-mpeg3",
	"audio/x-ogg",
	"audio/x-opus",
	"audio/x-vorbis",
	"audio/x-wav",
	"audio/x-wavpack",
	"audio/x-wavpack-correction",
];

/// TODO
///
/// Sorted for binary search.
pub const SUPPORTED_IMG_MIME_TYPES: [&str; 9] = [
	"image/bmp",
	"image/ico",
	"image/jpeg",
	"image/jpg",
	"image/png",
	"image/tiff",
	"image/vnd.microsoft.icon", // thanks microsoft.
	"image/webp",
	"image/x-icon",
];