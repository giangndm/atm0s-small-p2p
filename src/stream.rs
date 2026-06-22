use std::{fmt::Debug, marker::PhantomData};
use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::anyhow;
use serde::{Serialize, de::DeserializeOwned};
use tokio_util::codec::LengthDelimitedCodec;
use tokio_util::codec::{Decoder, Encoder};

use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug)]
pub struct P2pQuicStream {
    read: RecvStream,
    write: SendStream,
}

impl PartialEq for P2pQuicStream {
    fn eq(&self, other: &Self) -> bool {
        self.read.id() == other.read.id() && self.write.id() == other.write.id()
    }
}

impl Eq for P2pQuicStream {}

impl P2pQuicStream {
    pub fn new(read: RecvStream, write: SendStream) -> Self {
        Self { read, write }
    }

    pub(crate) fn write_stopped(&self) -> impl Future<Output = Result<Option<quinn::VarInt>, quinn::StoppedError>> + Send + Sync + 'static {
        self.write.stopped()
    }
}

impl AsyncRead for P2pQuicStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().read).poll_read(cx, buf)
    }
}

impl AsyncWrite for P2pQuicStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        let w: &mut (dyn AsyncWrite + Unpin) = &mut self.get_mut().write;
        Pin::new(w).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.get_mut().write).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.get_mut().write).poll_shutdown(cx)
    }
}

pub struct BincodeCodec<Item> {
    length_decode: LengthDelimitedCodec,
    _tmp: PhantomData<Item>,
}

impl<Item> BincodeCodec<Item> {
    pub(crate) fn with_max_frame_length(max: usize) -> Self {
        Self {
            length_decode: LengthDelimitedCodec::builder().max_frame_length(max).new_codec(),
            _tmp: Default::default(),
        }
    }
}

impl<Item> Default for BincodeCodec<Item> {
    fn default() -> Self {
        Self {
            length_decode: LengthDelimitedCodec::default(),
            _tmp: Default::default(),
        }
    }
}

impl<Item: Serialize> Encoder<Item> for BincodeCodec<Item> {
    type Error = std::io::Error;

    fn encode(&mut self, item: Item, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), Self::Error> {
        let data: Vec<u8> = bincode::serialize(&item).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "bincode serialize failure"))?;
        self.length_decode.encode(data.into(), dst)
    }
}

impl<Item: DeserializeOwned + Debug> Decoder for BincodeCodec<Item> {
    type Error = std::io::Error;
    type Item = Item;

    fn decode(&mut self, src: &mut tokio_util::bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.length_decode.decode(src)? {
            Some(buf) => Ok(Some(
                bincode::deserialize(&buf).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "bincode deserialize failure"))?,
            )),
            None => Ok(None),
        }
    }
}

pub async fn wait_object<R: AsyncRead + Unpin, O: DeserializeOwned, const MAX_SIZE: usize>(reader: &mut R) -> anyhow::Result<O> {
    let mut len_buf = [0; 2];
    let mut data_buf = [0; MAX_SIZE];
    reader.read_exact(&mut len_buf).await?;
    let handshake_len = u16::from_be_bytes([len_buf[0], len_buf[1]]) as usize;
    if handshake_len > data_buf.len() {
        return Err(anyhow!("packet to big {} vs {MAX_SIZE}", data_buf.len()));
    }

    reader.read_exact(&mut data_buf[0..handshake_len]).await?;

    Ok(bincode::deserialize(&data_buf[0..handshake_len])?)
}

pub async fn write_object<W: AsyncWrite + Send + Unpin, O: Serialize, const MAX_SIZE: usize>(writer: &mut W, object: &O) -> anyhow::Result<()> {
    let data_buf: Vec<u8> = bincode::serialize(object)?;
    if data_buf.len() > MAX_SIZE {
        return Err(anyhow!("buffer to big {} vs {MAX_SIZE}", data_buf.len()));
    }
    if data_buf.len() > u16::MAX as usize {
        return Err(anyhow!("buffer too big for u16 length prefix: {}", data_buf.len()));
    }
    let len_buf = (data_buf.len() as u16).to_be_bytes();

    writer.write_all(&len_buf).await?;
    writer.write_all(&data_buf).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use futures::FutureExt;
    use serde::{Serializer, ser::SerializeSeq};
    use tokio_util::{
        bytes::BytesMut,
        codec::{Decoder, Encoder},
    };

    use crate::{
        PeerId,
        msg::{P2pServiceId, PeerMessage},
    };

    use super::*;

    struct FailingSerialize;

    impl Serialize for FailingSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("intentional stream serialization failure"))
        }
    }

    struct GrowingSerialize {
        first_pass: Cell<bool>,
    }

    impl Serialize for GrowingSerialize {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let len = if self.first_pass.replace(false) {
                1
            } else {
                32
            };
            let mut seq = serializer.serialize_seq(Some(len))?;
            for _ in 0..len {
                seq.serialize_element(&7u8)?;
            }
            seq.end()
        }
    }

    #[test]
    fn peer_message_codec_must_reject_oversized_service_payloads() {
        let mut codec = BincodeCodec::<PeerMessage>::with_max_frame_length(60_000);
        let mut dst = BytesMut::new();
        let oversized = vec![0; 70_000];

        let result = codec.encode(PeerMessage::Unicast(PeerId::from(1), PeerId::from(2), P2pServiceId::from(0), oversized), &mut dst);

        assert!(result.is_err(), "main peer message codec must reject oversized service payloads before framing");
        assert!(dst.is_empty(), "oversized peer messages must not append partial frames");
    }

    #[test]
    fn peer_message_codec_must_reject_oversized_inbound_frames() {
        let mut codec = BincodeCodec::<PeerMessage>::with_max_frame_length(60_000);
        let mut src = BytesMut::new();
        src.extend_from_slice(&(70_000u32.to_be_bytes()));

        let result = codec.decode(&mut src);

        assert!(result.is_err(), "main peer message codec must reject oversized inbound frames before payload allocation");
    }

    #[test]
    fn peer_message_codec_must_allow_small_messages() {
        let mut codec = BincodeCodec::<PeerMessage>::with_max_frame_length(60_000);
        let mut framed = BytesMut::new();
        codec
            .encode(PeerMessage::Unicast(PeerId::from(1), PeerId::from(2), P2pServiceId::from(0), b"ok".to_vec()), &mut framed)
            .expect("small peer message should encode");

        let decoded = codec.decode(&mut framed).expect("small peer message should decode");

        assert!(
            matches!(decoded, Some(PeerMessage::Unicast(source, dest, service, payload)) if source == PeerId::from(1) && dest == PeerId::from(2) && service == P2pServiceId::from(0) && payload == b"ok")
        );
    }

    #[tokio::test]
    async fn write_object_must_return_error_on_serialize_failure() {
        let (mut writer, _reader) = tokio::io::duplex(1024);

        let result = std::panic::AssertUnwindSafe(write_object::<_, _, 1024>(&mut writer, &FailingSerialize)).catch_unwind().await;

        assert!(matches!(result, Ok(Err(_))), "write_object must return Err instead of panicking when serialization fails");
    }

    #[tokio::test]
    async fn write_object_must_reject_payloads_larger_than_u16_length_prefix() {
        let mut writer = Vec::new();
        let payload = vec![7u8; 70_000];

        let result = write_object::<_, _, 100_000>(&mut writer, &payload).await;

        assert!(result.is_err(), "write_object must reject objects larger than the two-byte length prefix can represent");
    }

    #[tokio::test]
    async fn write_object_must_recheck_actual_serialized_size() {
        let mut writer = Vec::new();
        let payload = GrowingSerialize { first_pass: Cell::new(true) };

        let result = write_object::<_, _, 16>(&mut writer, &payload).await;

        assert!(
            result.is_err(),
            "write_object must reject the actual serialized payload when it exceeds MAX_SIZE, not only the estimate"
        );
        assert!(
            writer.len() <= 18,
            "write_object must not write an oversized frame after a smaller serialized_size estimate, wrote {} bytes",
            writer.len()
        );
    }
}
