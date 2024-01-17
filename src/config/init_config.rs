//! TODO

//---------------------------------------------------------------------------------------------------- use
use std::marker::PhantomData;
use crate::{
	error::SansanError,
	config::{Callbacks,LiveConfig},
	engine::Engine,
	state::AudioState,
	extra_data::ExtraData,
};
use strum::{
	AsRefStr,
	Display,
	EnumCount,
	EnumString,
	EnumVariantNames,
	IntoStaticStr,
};

//---------------------------------------------------------------------------------------------------- InitConfig
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
/// TODO
pub struct InitConfig<Extra: ExtraData> {
	//------------------------------------------ Engine
	/// TODO
	pub callbacks: Callbacks,
	/// TODO
	pub callback_low_priority: bool,
	/// TODO
	pub init_blocking: bool,

	//------------------------------------------ Media Controls
	/// TODO
	pub media_controls: bool,

	//------------------------------------------ Restore state/settings
	/// TODO
	pub audio_state: Option<AudioState<Extra>>,
	/// TODO
	pub live_config: Option<LiveConfig>,
}

//---------------------------------------------------------------------------------------------------- InitConfig Impl
impl<Extra: ExtraData> InitConfig<Extra> {
	/// Return a reasonable default [`InitConfig`].
	///
	/// For the generics:
	/// - `Data`: 1st `()` means the [`AudioState`] will contain no extra data, or more accurately, `()` will be the extra data
	/// - `Call`: 2nd `()` means our callback channel (if even provided) will be the `()` channel, or more accurately, it will do nothing
	/// - `Error`: 3rd `()` means our error callback channel (if even provided) will also do nothing
	///
	/// This means, this [`InitConfig`] makes the [`Engine`]
	/// do nothing on channel-based callbacks, and will
	/// also not report any errors that occur, since that
	/// is also `()`.
	///
	/// Of course, you can (and probably should) override these generics,
	/// and provide any custom combination of `Extra, Call, Error`.
	///
	/// ```rust
	/// # use sansan::config::*;
	/// InitConfig::<()> {
	///     callbacks:             Callbacks::DEFAULT,
	///     callback_low_priority: true,
	///     init_blocking:         false,
	///     audio_state:           None,
	///     live_config:           None,
	///     media_controls:        false,
	/// };
	/// ```
	pub const DEFAULT: Self = Self {
		callbacks:             Callbacks::DEFAULT,
		callback_low_priority: true,
		init_blocking:         false,
		media_controls:        false,
		audio_state:           None,
		live_config:           None,
	};
}

impl<Extra: ExtraData> Default for InitConfig<Extra> {
	fn default() -> Self {
		Self::DEFAULT
	}
}
