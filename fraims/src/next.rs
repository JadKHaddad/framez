/// Calls [`Framed::maybe_next`](crate::Framed::maybe_next) in a loop until a frame is returned or an error occurs.
///
/// # Return value
///
/// - `Some(Ok(frame))` if a frame was successfully decoded. Call `next` again to read more frames.
/// - `Some(Err(error))` if an error occurred. The caller should stop reading.
/// - `None` if eof was reached. The caller should stop reading.
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
