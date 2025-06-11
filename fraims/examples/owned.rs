//! ```not_rust
//! cargo run --example owned
//! ```

use core::{error::Error, pin::pin};

use embedded_io_adapters::tokio_1::FromTokio;
use fraims::{
    FramedRead, FramedWrite,
    codec::lines::{StrLines, StringLines},
};
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("reader=info,writer=info")
        .init();

    let (read, write) = tokio::io::duplex(1024);

    let read_buf = &mut [0u8; 1024];
    let mut framed_read = FramedRead::new(StringLines::<32>::new(), FromTokio::new(read), read_buf);

    let reader = async move {
        let stream = framed_read.stream();
        let mut stream = pin!(stream);

        while let Some(item) = stream.next().await.transpose()? {
            tracing::info!(target: "reader", %item, "received frame")
        }

        Ok::<(), Box<dyn Error>>(())
    };

    let write_buf = &mut [0u8; 1024];
    let mut framed_write = FramedWrite::new(StrLines::new(), FromTokio::new(write), write_buf);

    let writer = async move {
        let items = ["Hello, world!", "How are you?", "Goodbye!"];

        let sink = framed_write.sink();
        let mut sink = pin!(sink);

        for item in items {
            tracing::info!(target: "writer", %item, "sending frame");

            sink.send(item).await?;
        }

        Ok::<(), Box<dyn Error>>(())
    };

    let (reader_result, writer_result) = tokio::join!(reader, writer);

    reader_result?;
    writer_result?;

    Ok(())
}
