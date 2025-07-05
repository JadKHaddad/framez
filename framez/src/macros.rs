/// Convenience macro to call [`maybe_next`](crate::functions::maybe_next) on a [`Framed`](crate::Framed) or [`FramedRead`](`crate::FramedRead`).
#[macro_export]
macro_rules! maybe_next {
    ($framed:expr) => {{
        $crate::functions::maybe_next(
            &mut $framed.core.state.read,
            &mut $framed.core.codec,
            &mut $framed.core.inner,
        )
        .await
    }};
}

/// Calls [`maybe_next!`](crate::maybe_next!) on a [`Framed`](crate::Framed) or [`FramedRead`](`crate::FramedRead`) in a loop until a frame is returned or an error occurs.
///
/// # Return value
///
/// - `Some(Ok(frame))` if a frame was successfully decoded. Call `next` again to read more frames.
/// - `Some(Err(error))` if an error occurred. The caller should stop reading.
/// - `None` if eof was reached. The caller should stop reading.
#[macro_export]
macro_rules! next {
    ($framed:expr) => {{
        'next: loop {
            match $crate::maybe_next!($framed) {
                Some(Ok(None)) => continue 'next,
                Some(Ok(Some(item))) => break 'next Some(Ok(item)),
                Some(Err(err)) => break 'next Some(Err(err)),
                None => break 'next None,
            }
        }
    }};
}

/// Convenience macro to call [`send`](crate::functions::send) on a [`Framed`](crate::Framed) or [`FramedWrite`](`crate::FramedWrite`).
#[macro_export]
macro_rules! send {
    ($framed:expr, $item:expr) => {{
        $crate::functions::send(
            &mut $framed.core.state.write,
            &mut $framed.core.codec,
            &mut $framed.core.inner,
            $item,
        )
        .await
    }};
}
