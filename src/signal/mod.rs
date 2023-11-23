//! [`Engine`] signals
#[allow(unused_imports)] // docs
use crate::Engine;

mod add;
pub use add::{Add,AddError,AddMany,AddManyError,InsertMethod};

mod back;
pub use back::{Back,BackError};

mod clear;
pub use clear::Clear;

mod next;
pub(crate) use next::Next;

mod pause;
pub(crate) use pause::Pause;

mod play;
pub(crate) use play::Play;

mod previous;
pub use previous::{Previous,PreviousError};

mod remove_range;
pub use remove_range::{RemoveRange,RemoveRangeError};

mod remove;
pub use remove::{Remove,RemoveError};

mod repeat;
pub use repeat::Repeat;
pub(crate) use repeat::AtomicRepeat;

mod seek;
pub use seek::{Seek,SeekError};
pub(crate) use seek::SetTime;

mod set_index;
pub use set_index::{SetIndex,SetIndexError};

mod shuffle;
pub use shuffle::Shuffle;

mod skip;
pub use skip::{Skip,SkipError};

mod stop;
pub(crate) use stop::Stop;

mod signal;
pub(crate) use signal::Signal;

mod toggle;
pub(crate) use toggle::Toggle;

mod volume;
pub use volume::Volume;
pub(crate) use volume::AtomicVolume;