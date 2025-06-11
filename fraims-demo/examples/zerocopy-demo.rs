//! ```not_rust
//! cargo run -p fraims-demo --example zerocopy-demo
//! ```

use core::error::Error;

use embedded_io_adapters::tokio_1::FromTokio;
use fraims::{FramedRead, FramedWrite, next};
use fraims_demo::{
    codec::PacketCodec,
    packet::Packet,
    payload_content::{DeviceConfig, DeviceConfigAck, Heartbeat, HeartbeatAck, Init, InitAck},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("reader=info,writer=info")
        .init();

    let (read, write) = tokio::io::duplex(1024);

    let read_buf = &mut [0u8; 1024];
    let mut framed_read = FramedRead::new(PacketCodec::new(), FromTokio::new(read), read_buf);

    let reader = async move {
        while let Some(packet) = next!(framed_read).transpose()? {
            tracing::info!(target: "reader", ?packet, "received packet")
        }

        Ok::<(), Box<dyn Error>>(())
    };

    let write_buf = &mut [0u8; 1024];
    let mut framed_write = FramedWrite::new(PacketCodec::new(), FromTokio::new(write), write_buf);

    let writer = async move {
        let packets = std::vec![
            Packet::new(Init {
                sequence_number: 0,
                version: "1.0.0",
            }),
            Packet::new(InitAck {
                sequence_number: 0,
                version: "1.0.0",
            }),
            Packet::new(Heartbeat { sequence_number: 1 }),
            Packet::new(HeartbeatAck { sequence_number: 1 }),
            Packet::new(DeviceConfig {
                sequence_number: 2,
                config: "very-important-config",
            }),
            Packet::new(DeviceConfigAck { sequence_number: 2 })
        ];

        for packet in packets {
            tracing::info!(target: "writer", ?packet, "sending packet");

            framed_write.send(packet).await?;
        }

        Ok::<(), Box<dyn Error>>(())
    };

    let (reader_result, writer_result) = tokio::join!(reader, writer);

    reader_result?;
    writer_result?;

    Ok(())
}
