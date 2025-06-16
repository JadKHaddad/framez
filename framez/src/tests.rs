#![allow(missing_docs)]

pub fn init_tracing() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::Subscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .finish(),
    )
    .ok();
}

macro_rules! framed_read {
    ($items:ident, $expected:ident, $decoder:ident) => {
        framed_read!($items, $expected, $decoder, 1024, 1024);
    };
    ($items:ident, $expected:ident, $decoder:ident, $buffer_size:literal) => {
        framed_read!($items, $expected, $decoder, $buffer_size, 1024);
    };
    ($items:ident, $expected:ident, $decoder:ident, $buffer_size:literal $(, $err:ident )?) => {
        framed_read!($items, $expected, $decoder, $buffer_size, 1024 $(, $err )?);
    };
    ($items:ident, $expected:ident, $decoder:ident, $buffer_size:literal, $duplex_max_size:literal $(, $err:ident )?) => {
        let decoder_clone = $decoder.clone();
        let mut collected = Vec::<Vec<u8>>::new();

        let (read, mut write) = tokio::io::duplex($duplex_max_size);

        tokio::spawn(async move {
            for item in $items {
                write.write_all(item.as_ref()).await.expect("Must write");
            }
        });

        let buffer = &mut [0_u8; $buffer_size];
        let mut framer =
            crate::FramedRead::new(decoder_clone, embedded_io_adapters::tokio_1::FromTokio::new(read), buffer);

        while let Some(item) = $crate::next!(framer) {
            match item {
                Ok(item) => {
                    collected.push(item.into());
                }
                Err(_err) => {
                    #[cfg(not(feature = "defmt"))]
                    crate::logging::error!("Error: {:?}", _err);

                    $(
                        assert!(matches!(_err, ReadError::$err));
                    )?

                    break;
                }
            }
        }

        assert_eq!($expected, collected);
    };
}

macro_rules! sink_stream {
    ($encoder:ident, $decoder:ident, $items:ident, $map:ident) => {
        let items_clone = $items.clone();

        let (read, write) = tokio::io::duplex(1024);

        tokio::spawn(async move {
            let buffer = &mut [0_u8; 1024];
            let mut writer = crate::FramedWrite::new(
                $encoder,
                embedded_io_adapters::tokio_1::FromTokio::new(write),
                buffer,
            );
            let sink = writer.sink();

            pin_mut!(sink);

            for item in items_clone.iter() {
                sink.send(item).await.expect("Must send");
            }
        });

        let buffer = &mut [0_u8; 1024];
        let mut framer = crate::FramedRead::new(
            $decoder,
            embedded_io_adapters::tokio_1::FromTokio::new(read),
            buffer,
        );

        let stream = framer.stream($map);

        let collected = stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!($items, collected);
    };
}

pub(crate) use framed_read;
pub(crate) use sink_stream;
