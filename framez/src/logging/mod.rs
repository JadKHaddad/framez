//! Logging utilities.

mod formatter;

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
pub(crate) use formatter::Formatter;

macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "tracing")]
        tracing::trace!(target: $target, $($arg)*);

        #[cfg(feature = "log")]
        log::trace!(target: $target, $($arg)*);

        #[cfg(feature = "defmt")]
        defmt::trace!($($arg)*);
    };
}

macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "tracing")]
        tracing::debug!(target: $target, $($arg)*);

        #[cfg(feature = "log")]
        log::debug!(target: $target, $($arg)*);

        #[cfg(feature = "defmt")]
        defmt::debug!($($arg)*);
    };
}

macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "tracing")]
        tracing::error!(target: $target, $($arg)*);

        #[cfg(feature = "log")]
        log::error!(target: $target, $($arg)*);

        #[cfg(feature = "defmt")]
        defmt::error!($($arg)*);
    };
}

macro_rules! warn_ {
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "tracing")]
        tracing::warn!(target: $target, $($arg)*);

        #[cfg(feature = "log")]
        log::warn!(target: $target, $($arg)*);

        #[cfg(feature = "defmt")]
        defmt::warn!($($arg)*);
    };
}

pub(crate) use debug;
pub(crate) use error;
pub(crate) use trace;
pub(crate) use warn_ as warn;
