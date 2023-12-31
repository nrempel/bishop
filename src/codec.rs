use bincode::{deserialize, serialize};
use bytes::BytesMut;
use bytes::{Buf, BufMut};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Message {
    Ping,
    Pong,
}

impl Message {
    pub fn response(&self) -> Option<Message> {
        match self {
            Message::Ping => Some(Message::Pong),
            Message::Pong => None,
        }
    }
}

#[derive(Debug)]
pub struct Codec;

impl Decoder for Codec {
    type Item = Message;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.len() >= MESSAGE_SIZE_OFFSET {
            let message_len = (&buf[..MESSAGE_SIZE_OFFSET]).get_u32() as usize;
            if buf.len() >= MESSAGE_SIZE_OFFSET + message_len {
                let msg_data = buf.split_to(MESSAGE_SIZE_OFFSET + message_len);
                let msg: Message = deserialize(&msg_data[MESSAGE_SIZE_OFFSET..])?;
                return Ok(Some(msg));
            }
        }
        Ok(None)
    }
}

impl Encoder<Message> for Codec {
    type Error = Error;

    fn encode(&mut self, item: Message, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = serialize(&item)?;
        buf.reserve(MESSAGE_SIZE_OFFSET + bytes.len());

        buf.put_u32(bytes.len() as u32);
        buf.put_slice(&bytes);
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("bincode error: {0}")]
    BincodeError(#[from] bincode::Error),
}

const MESSAGE_SIZE_OFFSET: usize = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_message() {
        let mut codec = Codec;
        let mut buf = BytesMut::new();
        let message = Message::Ping;

        let res_encode = codec.encode(message.clone(), &mut buf);
        let res_decode = codec.decode(&mut buf).unwrap().unwrap();

        assert!(res_encode.is_ok());
        assert_eq!(message, res_decode);
    }
}
