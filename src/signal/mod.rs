//! [`Engine`] signals
#[allow(unused_imports)] // docs
use crate::Engine;

pub(crate) mod add;
pub use add::{Add,AddMany,InsertMethod};

pub(crate) mod back;
pub use back::{Back,BackError};

pub(crate) mod clear;
pub use clear::Clear;

pub(crate) mod next;
pub(crate) use next::Next;

pub(crate) mod pause;
pub(crate) use pause::Pause;

pub(crate) mod play;
pub(crate) use play::Play;

pub(crate) mod previous;
pub use previous::{Previous,PreviousError};

pub(crate) mod remove_range;
pub(crate) use remove_range::RemoveRange;
// pub use remove_range::RemoveRangeError;

pub(crate) mod remove;
pub use remove::{Remove,RemoveError};

pub(crate) mod repeat;
pub use repeat::Repeat;
pub(crate) use repeat::AtomicRepeat;

pub(crate) mod seek;
pub use seek::{Seek,SeekError};
pub(crate) use seek::SeekedTime;

pub(crate) mod set_index;
pub use set_index::{SetIndex,SetIndexError};

pub(crate) mod shuffle;
pub use shuffle::Shuffle;

pub(crate) mod skip;
pub use skip::{Skip,SkipError};

pub(crate) mod stop;
pub(crate) use stop::Stop;

pub(crate) mod toggle;
pub(crate) use toggle::Toggle;

pub(crate) mod volume;
pub use volume::Volume;
pub(crate) use volume::AtomicVolume;