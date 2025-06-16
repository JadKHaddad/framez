//! ```not_rust
//! cargo run --example zerocopy --features tracing
//! ```

use core::error::Error;

use embedded_io_adapters::tokio_1::FromTokio;
use framez::{FramedRead, FramedWrite, codec::lines::StrLines, next};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("reader=info,writer=info,framez=trace")
        .init();

    let (read, write) = tokio::io::duplex(8);

    let read_buf = &mut [0u8; 1024];
    let mut framed_read = FramedRead::new(StrLines::new(), FromTokio::new(read), read_buf);

    let reader = async move {
        while let Some(item) = next!(framed_read).transpose()? {
            tracing::info!(target: "reader", item, "received frame")
        }

        Ok::<(), Box<dyn Error>>(())
    };

    let write_buf = &mut [0u8; 1024];
    let mut framed_write = FramedWrite::new(StrLines::new(), FromTokio::new(write), write_buf);

    let writer = async move {
        let items = ["Hello, world!", "How are you?", "Goodbye!"];

        for item in items {
            tracing::info!(target: "writer", item, "sending frame");

            framed_write.send(item).await?;
        }

        Ok::<(), Box<dyn Error>>(())
    };

    let (reader_result, writer_result) = tokio::join!(reader, writer);

    reader_result?;
    writer_result?;

    Ok(())
}
