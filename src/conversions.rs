use crate::model::{Packet, PacketType};

// Byte array to packet
impl Packet {
    pub fn from_u8_arr(buffer: &[u8]) -> anyhow::Result<Self> {
        let length_slice: [u8; 4] = buffer[0..4].try_into().unwrap();
        let request_id_slice: [u8; 4] = buffer[4..8].try_into().unwrap();
        let packet_type_slice: [u8; 4] = buffer[8..12].try_into().unwrap();
        let mut payload_vec: Vec<u8> = Vec::new();
        for item in buffer.iter().skip(12) {
            match item {
                0 => break,
                _ => payload_vec.push(*item),
            }
        }

        Ok(Self {
            length: Some(Self::parse_length(length_slice)),
            request_id: Self::parse_request_id(request_id_slice),
            packet_type: Self::parse_packet_type(packet_type_slice),
            payload: Self::parse_payload(payload_vec),
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
        Some(String::from_utf8(buffer).unwrap())
    }
}

// To byte array
impl Packet {
    pub fn into_vec(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // LENGTH (32 bit integer - 4 bytes)
        if self.length.is_none() {
            let mut length: i32 = 0;
            length += 4; // request id (i32 = 4 bytes)
            length += 4; // packet type (i32 = 4 bytes)
            length += self.payload.clone().unwrap_or_default().len() as i32 + 1; // Payload length + NULL-terminator (payload length + 1 byte)
            length += 1; // NULL-terminator (1 byte)
            buffer.extend_from_slice(&length.to_le_bytes());
        } else {
            buffer.extend_from_slice(&self.length.unwrap().to_le_bytes());
        }

        // REQUEST ID (32 bit integer - 4 bytes)
        let request_id_buf = self.request_id.to_le_bytes();
        buffer.extend_from_slice(&request_id_buf);

        // REQUEST TYPE (32 bit integer - 4 bytes)
        let request_type_buf = self.packet_type.as_i32().to_le_bytes();
        buffer.extend_from_slice(&request_type_buf);

        // PAYLOAD (00-terminated string)
        if self.payload.clone().is_some() {
            let payload_buf = self.payload.clone().unwrap();
            buffer.extend_from_slice(payload_buf.as_bytes());
        }
        buffer.push(0); // terminate string

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
            Self::Invalid(n) => n
        }
    }
}
