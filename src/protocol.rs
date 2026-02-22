use bytes::{Buf, BufMut};
use thiserror::Error;

// TMQP has Header size as 32 bytes
pub const HEADER_SIZE: usize = 32;
pub const MAGIC_NUMBER: u32 = 0x544D5150;


#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Buffer too small: needed {needed} bytes, but got {available}")]
    Incomplete {needed: usize, available: usize},
    #[error("Invalid Magic Number: expected 0x544D5150, got {0:#X}")]
    InvalidMagic(u32),
    #[error("Unknown Message Type: {0}")]
    UnknownMsgType(u8),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Connect = 1,
    Subscribe = 2,
    Publish = 3,
    TensorMeta = 4,
    TensorChunk = 5,
    Ack = 6,
    Heartbeat = 7,
    Error = 8,
}

impl TryFrom<u8> for MsgType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            1 => Ok(MsgType::Connect),
            2 => Ok(MsgType::Subscribe),
            3 => Ok(MsgType::Publish),
            4 => Ok(MsgType::TensorMeta),
            5 => Ok(MsgType::TensorChunk),
            6 => Ok(MsgType::Ack),
            7 => Ok(MsgType::Heartbeat),
            8 => Ok(MsgType::Error),
            _ => Err(ProtocolError::UnknownMsgType(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub version: u16,
    pub msg_type: MsgType,
    pub flags: u8,
    pub stream_id: u64,
    pub topic_len: u32,
    pub meta_len: u32,
    pub data_len:u64,
}

impl Header {
    /// Encode the header into a mutable buffer for network transmission.
    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u32(MAGIC_NUMBER);
        buf.put_u16(self.version);
        buf.put_u8(self.msg_type as u8);
        buf.put_u8(self.flags);
        buf.put_u64(self.stream_id);
        buf.put_u32(self.topic_len);
        buf.put_u32(self.meta_len);
        buf.put_u64(self.data_len);
    }

    /// Decodes a 32 byte header from a buffer. Validates the magic number and ensures enough bytes are present.
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, ProtocolError> {
        if buf.remaining() < HEADER_SIZE {
            return Err(ProtocolError::Incomplete {
                needed: HEADER_SIZE,
                available: buf.remaining(),
            });
        }

        let magic = buf.get_u32();
        if magic != MAGIC_NUMBER {
            return Err(ProtocolError::InvalidMagic(magic));
        }

        let version = buf.get_u16();
        let msg_type_raw = buf.get_u8();
        let flags = buf.get_u8();
        let stream_id = buf.get_u64();
        let topic_len = buf.get_u32();
        let meta_len = buf.get_u32();
        let data_len = buf.get_u64();

        let msg_type = MsgType::try_from(msg_type_raw)?;

        Ok(Header {
            version, msg_type, flags, stream_id, topic_len, meta_len, data_len,
        })
    }
}