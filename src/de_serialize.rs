use anyhow::bail;
use crate::model::{Packet, PacketType};

// Deserialize
impl Packet {
    pub fn try_deserialize(buffer: Vec<u8>) -> anyhow::Result<Self> {
        let length_slice: [u8; 4] = buffer[0..4].try_into()?;
        let packet_length = Self::parse_length(length_slice) as isize;
        if packet_length < 0 {
            bail!("Packet length is negative")
        }
        let packet_length = packet_length as usize;
        let raw_packet = &buffer[..packet_length+4];
        // Checks if the buffer length is equal to the packet length from the packet
        if buffer.len() != packet_length+4 {
            bail!("Wrong length (actual length [{}] != packet length [{}])", buffer.len(), packet_length+4)
        }
        // Checks if the packet is NULL-terminated
        if buffer[buffer.len()-1] != 0 {
            bail!("Packet was not NULL-terminated")
        }
        // Checks if the ASCII string is NULL-terminated
        if buffer[buffer.len()-2] != 0 {
            bail!("Payload was not NULL-terminated")
        }
        let request_id_slice: [u8; 4] = raw_packet[4..8].try_into()?;
        let packet_type_slice: [u8; 4] = raw_packet[8..12].try_into()?;
        let mut packet_iter = raw_packet.iter().skip(12);
        let mut payload_buf = vec![];
        while let Some(byte) = packet_iter.next() {
            if *byte == 0 {
                break
            }
            payload_buf.push(*byte)
        }

        Ok(Self {
            length: Some(Self::parse_length(length_slice)),
            request_id: Self::parse_request_id(request_id_slice),
            packet_type: Self::parse_packet_type(packet_type_slice),
            payload: Self::parse_payload(payload_buf),
        })
    }

    fn parse_length(buffer: [u8; 4]) -> i32 {
        i32::from_le_bytes(buffer)
    }

    fn parse_request_id(buffer: [u8; 4]) -> i32 {
        i32::from_le_bytes(buffer)
    }

    fn parse_packet_type(buffer: [u8; 4]) -> PacketType {
        let request_type = i32::from_le_bytes(buffer);
        match request_type {
            0 => PacketType::MultiPacketResponse,
            2 => PacketType::RunCommand,
            3 => PacketType::Login,
            _ => PacketType::Invalid(request_type),
        }
    }

    fn parse_payload(buffer: Vec<u8>) -> Option<String> {
        if buffer.is_empty() {
            return None;
        }
        Some(String::from_utf8(buffer.to_vec()).unwrap_or_else(|_| "(UTF-8 error)".to_string()))
    }
}

// Serialize
impl Packet {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // LENGTH (32 bit integer - 4 bytes)
        buffer.extend_from_slice(
            &self
                .length
                .map_or_else(
                    || {
                        let mut len: i32 = 0;
                        len += 4; // request id (i32 = 4 bytes)
                        len += 4; // packet type (i32 = 4 bytes)
                        len += self.payload.clone().unwrap_or_default().len() as i32 + 1; // Payload length + NULL-terminator (payload length + 1 byte)
                        len + 1 // NULL-terminator (1 byte)
                    },
                    |len| len,
                )
                .to_le_bytes(),
        );

        // REQUEST ID (32 bit integer - 4 bytes)
        let request_id_buf = self.request_id.to_le_bytes();
        buffer.extend_from_slice(&request_id_buf);

        // REQUEST TYPE (32 bit integer - 4 bytes)
        let request_type_buf = self.packet_type.as_i32().to_le_bytes();
        buffer.extend_from_slice(&request_type_buf);

        // PAYLOAD (00-terminated string)
        if let Some(pl) = self.payload.clone() {
            buffer.extend_from_slice(pl.as_bytes());
        }

        // terminate string
        buffer.push(0);

        // NULL-termination
        buffer.push(0);
        buffer.to_vec()
    }
}

impl PacketType {
    pub fn as_i32(&self) -> &i32 {
        match self {
            Self::Login => &3,
            Self::Auth | Self::RunCommand => &2,
            Self::MultiPacketResponse => &0,
            Self::Invalid(n) => n,
        }
    }
}
