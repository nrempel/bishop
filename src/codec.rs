use bytes::{Buf, BufMut, BytesMut};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub(super) struct Codec;

impl Decoder for Codec {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match buf.iter().position(|&b| b == b'\n') {
            Some(i) => {
                let line = buf.split_to(i);
                buf.advance(1);
                let s = std::str::from_utf8(&line)?;
                Ok(Some(s.to_string()))
            }
            None => Ok(None),
        }
    }
}

impl Encoder<String> for Codec {
    type Error = Error;

    fn encode(&mut self, data: String, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put(data.as_bytes());
        buf.put_u8(b'\n');
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid string: {0}")]
    InvalidStringUtf8(#[from] std::str::Utf8Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}
