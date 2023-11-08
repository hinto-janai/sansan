// Global macros for internal sansan usage.

// Receive a channel message, unwrap.
macro_rules! recv {
    ($channel:expr) => {
        $channel.recv().unwrap()
    }
}
pub(crate) use recv;

// Send a channel message, unwrap.
macro_rules! send {
    ($channel:expr, $($msg:tt)*) => {
        $channel.send($($msg)*).unwrap()
    }
}
pub(crate) use send;
