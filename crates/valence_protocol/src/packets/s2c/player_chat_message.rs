use std::io::Write;

use crate::{Decode, DecodePacket, Encode, EncodePacket, Text, Uuid, VarInt};

#[derive(Clone, PartialEq, Debug, EncodePacket, DecodePacket)]
#[packet_id = 0x31]
pub struct PlayerChatMessage<'a> {
    pub sender: Uuid,
    pub index: VarInt,
    pub message_signature: Option<&'a [u8; 256]>,
    pub message: String,
    pub time_stamp: u64,
    pub salt: u64,
    pub previous_messages: Vec<PreviousMessage<'a>>,
    pub unsigned_content: Option<Text>,
    pub filter_type: MessageFilterType,
    pub filter_type_bits: Option<u8>,
    pub chat_type: VarInt,
    pub network_name: Text,
    pub network_target_name: Option<Text>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PreviousMessage<'a> {
    pub message_id: i32,
    pub signature: Option<&'a [u8; 256]>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum MessageFilterType {
    PassThrough,
    FullyFiltered,
    PartiallyFiltered,
}

impl<'a> Encode for PlayerChatMessage<'a> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        self.sender.encode(&mut w)?;
        self.index.encode(&mut w)?;
        self.message_signature.encode(&mut w)?;
        self.message.encode(&mut w)?;
        self.time_stamp.encode(&mut w)?;
        self.salt.encode(&mut w)?;
        self.previous_messages.encode(&mut w)?;
        self.unsigned_content.encode(&mut w)?;
        self.filter_type.encode(&mut w)?;

        if self.filter_type == MessageFilterType::PartiallyFiltered {
            match self.filter_type_bits {
                // Filler data
                None => 0u8.encode(&mut w)?,
                Some(bits) => bits.encode(&mut w)?,
            }
        }

        self.chat_type.encode(&mut w)?;
        self.network_name.encode(&mut w)?;
        self.network_target_name.encode(&mut w)?;

        Ok(())
    }
}

impl<'a> Decode<'a> for PlayerChatMessage<'a> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let sender = Uuid::decode(r)?;
        let index = VarInt::decode(r)?;
        let message_signature = Option::<&'a [u8; 256]>::decode(r)?;
        let message = String::decode(r)?;
        let time_stamp = u64::decode(r)?;
        let salt = u64::decode(r)?;
        let previous_messages = Vec::<PreviousMessage>::decode(r)?;
        let unsigned_content = Option::<Text>::decode(r)?;
        let filter_type = MessageFilterType::decode(r)?;

        let filter_type_bits = match filter_type {
            MessageFilterType::PartiallyFiltered => Some(u8::decode(r)?),
            _ => None,
        };

        let chat_type = VarInt::decode(r)?;
        let network_name = Text::decode(r)?;
        let network_target_name = Option::<Text>::decode(r)?;

        Ok(Self {
            sender,
            index,
            message_signature,
            message,
            time_stamp,
            salt,
            previous_messages,
            unsigned_content,
            filter_type,
            filter_type_bits,
            chat_type,
            network_name,
            network_target_name,
        })
    }
}

impl<'a> Encode for PreviousMessage<'a> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        VarInt(self.message_id + 1).encode(&mut w)?;

        match self.signature {
            None => {}
            Some(signature) => signature.encode(&mut w)?,
        }

        Ok(())
    }
}

impl<'a> Decode<'a> for PreviousMessage<'a> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let message_id = VarInt::decode(r)?.0 - 1;

        let signature = if message_id == -1 {
            Some(<&[u8; 256]>::decode(r)?)
        } else {
            None
        };

        Ok(Self {
            message_id,
            signature,
        })
    }
}
