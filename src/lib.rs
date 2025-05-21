#![no_std]
#![deny(unsafe_code)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codec;
pub mod decode;
pub mod encode;

mod read;
pub use read::{ReadError, ReadFrames};

mod write;
pub use write::{WriteError, WriteFrames};

pub(crate) mod logging;

#[macro_export]
macro_rules! next {
    ($framed:ident) => {{
        'next: loop {
            match $framed.maybe_next().await {
                Some(Ok(None)) => continue 'next,
                Some(Ok(Some(item))) => break 'next Some(Ok(item)),
                Some(Err(err)) => break 'next Some(Err(err)),
                None => break 'next None,
            }
        }
    }};
}
