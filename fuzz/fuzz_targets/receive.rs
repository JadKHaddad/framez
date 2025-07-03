//! If we panic!, we lose.
//!
//! ```not_rust
//! cargo +nightly fuzz run receive
//! ```

#![no_main]

use std::error::Error;

use embedded_io_adapters::tokio_1::FromTokio;
use framez::{codec::lines::StrLines, next, FramedRead};
use libfuzzer_sys::fuzz_target;
use tokio::{io::AsyncWriteExt, runtime::Runtime};

fuzz_target!(|data: &[u8]| {
    Runtime::new().expect("Runtime must build").block_on(async {
        fuzz(data).await.unwrap();
    });
});

const SIZE: usize = 1024;

async fn fuzz(data: &[u8]) -> Result<(), Box<dyn Error>> {
    if data.is_empty() {
        return Ok(());
    }

    let (read, mut write) = tokio::io::duplex(32);

    let read_buf = &mut [0u8; SIZE];

    let mut framed_read = FramedRead::new(StrLines::new(), FromTokio::new(read), read_buf);

    let reader = async move {
        let _ = next!(framed_read);

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
