//! TODO

//---------------------------------------------------------------------------------------------------- use
use std::marker::PhantomData;
use crate::{
	error::SansanError,
	config::callbacks::Callbacks,
	engine::Engine,
	state::AudioState,
	valid_data::ValidData,
};
use strum::{
	AsRefStr,
	Display,
	EnumCount,
	EnumString,
	EnumVariantNames,
	IntoStaticStr,
};

//---------------------------------------------------------------------------------------------------- Config
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
/// TODO
pub struct Config<Data>
where
	Data: ValidData,
{
	//------------------------------------------ Engine
	/// TODO
	pub callbacks: Callbacks,
	/// TODO
	pub callback_low_priority: bool,
	/// TODO
	pub init_blocking: bool,
	/// TODO
	pub shutdown_blocking: bool,

	//------------------------------------------ Audio
	/// TODO
	pub restore: Option<AudioState<Data>>,
	/// TODO
	pub back_threshold: f64,

	//------------------------------------------ Media Controls
	/// TODO
	pub media_controls: bool,
}

//---------------------------------------------------------------------------------------------------- Config Impl
impl<Data> Config<Data>
where
	Data: ValidData,
{
	/// Return a reasonable default [`Config`].
	///
	/// For the generics:
	/// - `Data`: 1st `()` means the [`AudioState`] will contain no extra data, or more accurately, `()` will be the extra data
	/// - `Call`: 2nd `()` means our callback channel (if even provided) will be the `()` channel, or more accurately, it will do nothing
	/// - `Error`: 3rd `()` means our error callback channel (if even provided) will also do nothing
	///
	/// This means, this [`Config`] makes the [`Engine`]
	/// do nothing on channel-based callbacks, and will
	/// also not report any errors that occur, since that
	/// is also `()`.
	///
	/// Of course, you can (and probably should) override these generics,
	/// and provide any custom combination of `Data, Call, Error`.
	///
	/// ```rust
	/// # use sansan::config::*;
	/// Config::<()> {
	///     callbacks:             Callbacks::DEFAULT,
	///     callback_low_priority: true,
	///     init_blocking:         false,
	///     shutdown_blocking:     false,
	///     restore:               None,
	///     back_threshold:    3.0,
	///     media_controls:        false,
	/// };
	/// ```
	pub const DEFAULT: Self = Self {
		callbacks:             Callbacks::DEFAULT,
		callback_low_priority: true,
		init_blocking:         false,
		shutdown_blocking:     false,
		restore:               None,
		back_threshold:        3.0,
		media_controls:        false,
	};
}

impl<Data: ValidData> Default for Config<Data> {
	fn default() -> Self {
		Self::DEFAULT
	}
}
