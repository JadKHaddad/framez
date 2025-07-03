//! If we panic!, we lose.
//!
//! ```not_rust
//! cargo +nightly fuzz run receive
//! ```

#![no_main]

use std::error::Error;

use embedded_io_adapters::tokio_1::FromTokio;
use framez::{codec::lines::StrLines, next_echoed, Echo, Framed};
use libfuzzer_sys::fuzz_target;
use tokio::{io::AsyncWriteExt, runtime::Runtime};

fn echo(data: &str) -> Echo<&str> {
    if data.len() % 2 == 0 {
        return Echo::Echo(data);
    }

    Echo::NoEcho(data)
}

fn no_echo(data: &str) -> Echo<&str> {
    Echo::NoEcho(data)
}

fuzz_target!(|data: &[u8]| {
    Runtime::new().expect("Runtime must build").block_on(async {
        fuzz(data, no_echo).await.unwrap();
        fuzz(data, echo).await.unwrap();
    });
});

const SIZE: usize = 1024 * 1024;

async fn fuzz<F>(data: &[u8], echo: F) -> Result<(), Box<dyn Error>>
where
    F: Copy + FnOnce(&str) -> Echo<&str>,
{
    if data.is_empty() {
        return Ok(());
    }

    let (read, mut write) = tokio::io::duplex(32);

    let read_buf = &mut [0u8; SIZE];

    let mut framed_read = Framed::new(StrLines::new(), FromTokio::new(read), read_buf, &mut []);

    let reader = async move {
        let _ = next_echoed!(framed_read, echo);

        Ok::<(), Box<dyn Error>>(())
    };

    let writer = async move {
        for chunk in data.to_vec().chunks(SIZE / 3) {
            write.write_all(chunk).await?;
            write.flush().await?;
        }

        Ok::<(), Box<dyn Error>>(())
    };

    let (reader_result, writer_result) = tokio::join!(reader, writer);

    reader_result?;

    // Ignore writer errors, as they are expected when the reader closes. (BrokenPipe)
    let _ = writer_result;

    Ok(())
}
