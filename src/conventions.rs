use crate::model::{Packet, PacketType};

// Byte array to packet
impl Packet {
    pub fn from_u8_arr(buffer: &[u8]) -> Result<Packet, failure::Error> {
        let length_slice: [u8; 4] = buffer[0..4].try_into().unwrap();
        let request_id_slice: [u8; 4] = buffer[4..8].try_into().unwrap();
        let packet_type_slice: [u8; 4] = buffer[8..12].try_into().unwrap();
        let mut payload_vec: Vec<u8> = Vec::new();
        for i in 12.. {
            match buffer[i] {
                0 => break,
                _ => payload_vec.push(buffer[i]),
            }
        }

        Ok(Packet {
            length: Some(Self::parse_length(length_slice)),
            request_id: Self::parse_request_id(request_id_slice),
            packet_type: Self::parse_packet_type(packet_type_slice)?,
            payload: Self::parse_payload(payload_vec),
        })
    }

    fn parse_length(buffer: [u8; 4]) -> i32 {
        i32::from(buffer[0])
    }

    fn parse_request_id(buffer: [u8; 4]) -> i32 {
        buffer[0] as i32
    }

    fn parse_packet_type(buffer: [u8; 4]) -> Result<PacketType, failure::Error> {
        let request_type = buffer[0];
        match request_type {
            0 => Ok(PacketType::MultiPacketResponse),
            2 => Ok(PacketType::RunCommand),
            3 => Ok(PacketType::Login),
            _ => Err(failure::err_msg("Invalid request type")),
        }
    }

    fn parse_payload(buffer: Vec<u8>) -> Option<String> {
        if buffer.len() == 0 {
            return None;
        }
        Some(String::from_utf8(buffer).unwrap())
    }
}

// To byte array
impl Packet {
    pub fn to_u8_arr(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // LENGTH (32 bit integer - 4 bytes)
        if self.length.is_none() {
            let mut length: i32 = 0;
            length += 4; // 4 bytes for request id
            length += 4; // 4 bytes for packet type
            length += self.payload.clone().unwrap_or(String::new()).len() as i32 + 1; // Payload length + NULL-terminator
            length += 1; // NULL-terminator
            buffer.extend_from_slice(&length.to_le_bytes());
        } else {
            buffer.extend_from_slice(&self.length.unwrap().to_le_bytes())
        }

        // REQUEST ID (32 bit integer - 4 bytes)
        let request_id_buf = self.request_id.to_le_bytes();
        buffer.extend_from_slice(&request_id_buf);

        // REQUEST TYPE (32 bit integer - 4 bytes)
        let request_type_buf = self.packet_type.to_i32().to_le_bytes();
        buffer.extend_from_slice(&request_type_buf);

        // PAYLOAD (00-terminated string)
        if self.payload.clone().is_some() {
            let payload_buf = self.payload.unwrap();
            buffer.extend_from_slice(payload_buf.as_bytes());
        }
        buffer.push(0); // terminate string

        // NULL-termination
        buffer.push(0);
        buffer
    }
}

impl PacketType {
    pub fn to_i32(&self) -> i32 {
        match self {
            PacketType::Login => 3,
            PacketType::Auth => 2,
            PacketType::MultiPacketResponse => 0,
            PacketType::RunCommand => 2,
        }
    }
}
