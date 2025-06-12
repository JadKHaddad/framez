//! Packet codec implementation module.

use framez::{
    decode::{DecodeError, Decoder},
    encode::Encoder,
};

use super::packet::{Packet, PacketFromSliceError, PacketWriteError};

/// A codec for encoding and decoding packets.
#[derive(Debug, Default)]
pub struct PacketCodec {}

impl PacketCodec {
    /// Creates a new [`PacketCodec`].
    pub fn new() -> Self {
        Default::default()
    }
}

impl DecodeError for PacketCodec {
    type Error = PacketFromSliceError;
}

impl<'buf> Decoder<'buf> for PacketCodec {
    type Item = Packet<'buf>;

    fn decode(&mut self, src: &'buf mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        Packet::maybe_packet_from_prefix(src)
    }
}

impl<'buf> Encoder<Packet<'buf>> for PacketCodec {
    type Error = PacketWriteError;

    fn encode(&mut self, item: Packet<'buf>, dst: &mut [u8]) -> Result<usize, Self::Error> {
        item.write_to(dst)
    }
}

#[cfg(test)]
mod test {
    use embedded_io_adapters::tokio_1::FromTokio;
    use framez::{FramedRead, next};
    use tokio::io::AsyncWriteExt;

    use crate::{
        payload_content::{DeviceConfig, DeviceConfigAck, Heartbeat, HeartbeatAck, Init, InitAck},
        tests::init_tracing,
    };

    use super::*;

    #[tokio::test]
    async fn encode_decode() {
        init_tracing();

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

        let decoder = PacketCodec::new();
        let mut encoder = PacketCodec::new();

        let (read, mut write) = tokio::io::duplex(8);

        let packets_clone = packets.clone();
        tokio::spawn(async move {
            let mut write_buf = [0; 512];

            for packet in packets_clone.into_iter() {
                let packet_length = encoder.encode(packet, &mut write_buf).expect("Must encode");

                write
                    .write_all(&write_buf[..packet_length])
                    .await
                    .expect("Must write");
            }
        });

        let mut read_buf = [0; 512];
        let mut framer = FramedRead::new(decoder, FromTokio::new(read), &mut read_buf);

        let mut index = 0;

        while let Some(packet) = next!(framer) {
            match packet {
                Ok(packet) => {
                    tracing::info!(?packet, "Packet");

                    // Can't move out of `framer`!
                    let expected = packets.get(index).expect("Must have packet");

                    assert_eq!(expected, &packet);

                    index += 1;
                }
                Err(err) => {
                    tracing::error!(?err, "Error");

                    panic!("Error {err:?}");
                }
            }
        }
    }
}
