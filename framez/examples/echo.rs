//! This example shows how to reuse the read and write buffers without splitting the `Framed` instance.
//!
//! ```not_rust
//! cargo run --example echo
//! ```

use core::error::Error;

use embedded_io_adapters::tokio_1::FromTokio;
use framez::{Framed, codec::lines::StrLines, next, send};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("server=info,client=info")
        .init();

    let (server, client) = tokio::io::duplex(16);

    let read_buf = &mut [0u8; 1024];
    let write_buf = &mut [0u8; 1024];
    let mut server = Framed::new(StrLines::new(), FromTokio::new(server), read_buf, write_buf);

    let server = async move {
        while let Some(item) = next!(server).transpose()? {
            tracing::info!(target: "server", item, "received frame");

            // echo the item back
            send!(server, item)?;

            if item == "Close" {
                tracing::info!(target: "server", "closing connection");

                break;
            }
        }

        Ok::<(), Box<dyn Error>>(())
    };

    let read_buf = &mut [0u8; 1024];
    let write_buf = &mut [0u8; 1024];
    let mut client = Framed::new(StrLines::new(), FromTokio::new(client), read_buf, write_buf);

    let client = async move {
        let items = ["Hello, world!", "How are you?", "Goodbye!", "Close"];

        for item in items {
            tracing::info!(target: "client", item, "sending frame");

            client.send(item).await?;
        }

        while let Some(item) = next!(client).transpose()? {
            tracing::info!(target: "client", item, "received frame");
        }

        Ok::<(), Box<dyn Error>>(())
    };

    let (server_result, client_result) = tokio::join!(server, client);

    server_result?;
    client_result?;

    Ok(())
}
