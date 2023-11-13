// Global macros for internal sansan usage.

//---------------------------------------------------------------------------------------------------- Channels
// Receive a channel message, unwrap.
macro_rules! recv {
    ($channel:expr) => {
        $channel.recv().unwrap()
    }
}
pub(crate) use recv;

// Send a channel message, unwrap.
macro_rules! send {
    ($channel:expr, $($msg:tt)+) => {
        $channel.send($($msg)+).unwrap()
    }
}
pub(crate) use send;

//---------------------------------------------------------------------------------------------------- Logging
// Logs with `log` but only if in debug
// mode or if the log feature is enabled.

macro_rules! error2 {
    ($($arg:tt)+) => {{
        #[cfg(feature = "log")]
        ::log::error!($($arg)+);
    }};
}
pub(crate) use error2;

macro_rules! warn2 {
    ($($arg:tt)+) => {{
        #[cfg(feature = "log")]
        ::log::warn!($($arg)+);
    }};
}
pub(crate) use warn2;

macro_rules! info2 {
    ($($arg:tt)+) => {{
        #[cfg(feature = "log")]
        ::log::info!($($arg)+);
    }};
}
pub(crate) use info2;

macro_rules! debug2 {
    ($($arg:tt)+) => {{
        #[cfg(feature = "log")]
        ::log::debug!($($arg)+);
    }};
}
pub(crate) use debug2;

macro_rules! trace2 {
    ($($arg:tt)+) => {{
        #[cfg(feature = "log")]
        ::log::trace!($($arg)+);
    }};
}
pub(crate) use trace2;