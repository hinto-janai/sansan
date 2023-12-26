// Global macros for internal sansan usage.

//---------------------------------------------------------------------------------------------------- Channels
// SAFETY:
// These macros are used in situations where
// a [send/recv] erroring is a logical error.

// Receive a channel message, unwrap.
macro_rules! recv {
    ($channel:expr) => {{
		$channel.recv().unwrap()
	}}
}
pub(crate) use recv;

// Send a channel message, unwrap.
macro_rules! send {
	($channel:expr, $($msg:tt)+) => {{
		$channel.send($($msg)+).unwrap()
	}}
}
pub(crate) use send;

// `try_send` a channel message, unwrap.
macro_rules! try_send {
    ($channel:expr, $($msg:tt)+) => {{
        $channel.try_send($($msg)+).unwrap()
    }}
}
pub(crate) use try_send;

// `try_recv` a channel message, unwrap.
macro_rules! try_recv {
    ($channel:expr) => {{
        $channel.try_recv().unwrap()
    }}
}
pub(crate) use try_recv;

// `recv` a [Select] operation channel message.
//
// These select operations get triggered spuriously,
// so we have to make sure something was actually
// sent to the channel, else, we [continue] in
// whatever loop we are in.
macro_rules! select_recv {
	($channel:expr) => {{
		match $channel.try_recv() {
			Ok(msg) => msg,
			_ => continue,
		}
	}}
}
pub(crate) use select_recv;

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