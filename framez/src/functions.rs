//! Utility functions for reading and writing frames.
//!
//! This module provides functions to read and write frames by decoupling the read and write states.
//! It is meant to be used for sending the read bytes back through the same stream without splitting the `Framed` instance.
//! This is useful for echo servers or protocol implementations that perform automatic responses while decoding frames.
//!
//! E.g. the websockets protocol requires to respond to the `ping` frame with a `pong` frame with the same payload.

use embedded_io_async::{Read, Write};

use crate::{
    ReadError, WriteError,
    decode::Decoder,
    encode::Encoder,
    logging::{debug, error, trace, warn},
    state::{ReadState, WriteState},
};

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
use crate::logging::Formatter;

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
const READ: &str = "framez::read";

#[cfg(any(feature = "log", feature = "defmt", feature = "tracing"))]
const WRITE: &str = "framez::write";

/// Tries to read a frame.
///
/// # Return value
///
/// - `Some(Ok(None))` if the buffer is not framable. Call `maybe_next` again to read more bytes.
/// - `Some(Ok(Some(frame)))` if a frame was successfully decoded. Call `maybe_next` again to read more bytes.
/// - `Some(Err(error))` if an error occurred. The caller should stop reading.
/// - `None` if eof was reached. The caller should stop reading.
///
/// # Usage
///
/// See [`next!`](crate::next!).
pub async fn maybe_next<'buf, C, R>(
    state: &'buf mut ReadState<'_>,
    codec: &mut C,
    read: &mut R,
) -> Option<Result<Option<C::Item>, ReadError<R::Error, C::Error>>>
where
    C: Decoder<'buf>,
    R: Read,
{
    trace!(target: READ, "maybe_next called");

    debug!(
        target: READ,
        "total_consumed: {}, index: {}, buffer: {:?}",
        state.total_consumed,
        state.index,
        Formatter(&state.buffer[state.total_consumed..state.index])
    );

    if state.shift {
        state
            .buffer
            .copy_within(state.total_consumed..state.index, 0);

        state.index -= state.total_consumed;
        state.total_consumed = 0;

        trace!(target: READ, "Buffer shifted. copied: {}", state.framable());

        state.shift = false;

        return Some(Ok(None));
    }

    if state.is_framable {
        if state.eof {
            trace!(target: READ, "Framing on EOF");

            match codec.decode_eof(&mut state.buffer[state.total_consumed..state.index]) {
                Ok(Some((item, size))) => {
                    state.total_consumed += size;

                    debug!(
                        target: READ,
                        "Frame decoded, consumed: {}, total_consumed: {}",
                        size, state.total_consumed,
                    );

                    return Some(Ok(Some(item)));
                }
                Ok(None) => {
                    debug!(target: READ, "No frame decoded");

                    state.is_framable = false;

                    if state.index != state.total_consumed {
                        error!(target: READ, "Bytes remaining on stream");

                        return Some(Err(ReadError::BytesRemainingOnStream));
                    }

                    return None;
                }
                Err(err) => {
                    error!(target: READ, "Failed to decode frame");

                    return Some(Err(ReadError::Decode(err)));
                }
            };
        }

        trace!(target: READ, "Framing");

        let buf_len = state.buffer.len();

        match codec.decode(&mut state.buffer[state.total_consumed..state.index]) {
            Ok(Some((item, size))) => {
                state.total_consumed += size;

                debug!(
                    target: READ,
                    "Frame decoded, consumed: {}, total_consumed: {}",
                    size, state.total_consumed,
                );

                return Some(Ok(Some(item)));
            }
            Ok(None) => {
                debug!(target: READ, "No frame decoded");

                state.shift = state.index >= buf_len;

                state.is_framable = false;

                return Some(Ok(None));
            }
            Err(err) => {
                error!(target: READ, "Failed to decode frame");

                return Some(Err(ReadError::Decode(err)));
            }
        }
    }

    if state.index >= state.buffer.len() {
        error!(target: READ, "Buffer too small");

        return Some(Err(ReadError::BufferTooSmall));
    }

    trace!(target: READ, "Reading");

    match read.read(&mut state.buffer[state.index..]).await {
        Err(err) => {
            error!(target: READ, "Failed to read");

            Some(Err(ReadError::IO(err)))
        }
        Ok(0) => {
            warn!(target: READ, "Got EOF");

            state.eof = true;

            state.is_framable = true;

            Some(Ok(None))
        }
        Ok(n) => {
            debug!(target: READ, "Bytes read. bytes: {}", n);

            state.index += n;

            state.is_framable = true;

            Some(Ok(None))
        }
    }
}

/// Like [`maybe_next`], but maps the decoded item to another type using the provided `map` function.
///
/// The output type `U` is static. This means it is decoupled from the lifetime of the [`ReadState`].
pub async fn maybe_next_mapped<'buf, C, R, U>(
    state: &'buf mut ReadState<'_>,
    codec: &mut C,
    read: &mut R,
    map: fn(<C as Decoder<'_>>::Item) -> U,
) -> Option<Result<Option<U>, ReadError<R::Error, C::Error>>>
where
    U: 'static,
    C: for<'a> Decoder<'a>,
    R: Read,
{
    match maybe_next(state, codec, read).await {
        Some(Ok(Some(item))) => Some(Ok(Some(map(item)))),
        Some(Ok(None)) => Some(Ok(None)),
        Some(Err(err)) => Some(Err(err)),
        None => None,
    }
}

/// Tries to read a frame and converts it using the given `map` function.
///
/// # Return value
///
/// - `Some(Ok(U))` if a frame was successfully decoded and mapped. Call `next` again to read more frames.
/// - `Some(Err(error))` if an error occurred. The caller should stop reading.
/// - `None` if eof was reached. The caller should stop reading.
pub async fn next<'buf, C, R, U>(
    state: &'buf mut ReadState<'_>,
    codec: &mut C,
    read: &mut R,
    map: fn(<C as Decoder<'_>>::Item) -> U,
) -> Option<Result<U, ReadError<R::Error, C::Error>>>
where
    U: 'static,
    C: for<'a> Decoder<'a>,
    R: Read,
{
    loop {
        match maybe_next_mapped(state, codec, read, map).await {
            Some(Ok(None)) => continue,
            Some(Ok(Some(item))) => return Some(Ok(item)),
            Some(Err(err)) => return Some(Err(err)),
            None => return None,
        }
    }
}

/// Sends a frame.
pub async fn send<C, W, I>(
    state: &mut WriteState<'_>,
    codec: &mut C,
    write: &mut W,
    item: I,
) -> Result<(), WriteError<W::Error, C::Error>>
where
    C: Encoder<I>,
    W: Write,
{
    match codec.encode(item, state.buffer) {
        Ok(size) => match write.write_all(&state.buffer[..size]).await {
            Ok(_) => {
                trace!(target: WRITE, "Wrote. buffer: {:?}", Formatter(&state.buffer[..size]));

                match write.flush().await {
                    Ok(_) => {
                        debug!(target: WRITE, "Flushed. bytes: {}", size);

                        Ok(())
                    }
                    Err(err) => {
                        error!(target: WRITE, "Failed to flush");

                        Err(WriteError::IO(err))
                    }
                }
            }
            Err(err) => {
                error!(target: WRITE, "Failed to write frame");

                Err(WriteError::IO(err))
            }
        },
        Err(err) => {
            error!(target: WRITE, "Failed to encode frame");

            Err(WriteError::Encode(err))
        }
    }
}
