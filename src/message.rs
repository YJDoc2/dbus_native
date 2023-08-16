// see https://dbus.freedesktop.org/doc/dbus-specification.html and
// https://dbus.freedesktop.org/doc/api/html/structDBusHeader.html

use crate::utils::adjust_padding;

// NOTE that we do not support all of the possible values and options, only those
// which are relevant and used by youki
pub enum MessageKind {
    MethodCall,
    MethodReply,
}

pub enum HeaderFieldKind {
    Path,
    Interface,
    Member,
    Destination,
    ErrorName,
    Sender,
    BodySignature,
}

pub struct Header {
    pub kind: HeaderFieldKind,
    pub value: String,
}
pub struct Message {
    pub kind: MessageKind,
    pub id: u32,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

// serialize without padding
fn serialize_headers(headers: &[Header]) -> Vec<u8> {
    let mut ret = vec![];
    for header in headers {
        let mut temp = vec![];
        adjust_padding(&mut ret, 8);
        // let required_padding = (8 - (ret.len() % 8)) % 8;

        // for _ in 0..required_padding {
        //     temp.push(0);
        // }

        let header_kind: u8 = match &header.kind {
            HeaderFieldKind::Path => 1,
            HeaderFieldKind::Interface => 2,
            HeaderFieldKind::Member => 3,
            HeaderFieldKind::ErrorName => 4,
            HeaderFieldKind::Destination => 6,
            HeaderFieldKind::Sender => 7,
            HeaderFieldKind::BodySignature => 8,
        };

        let header_signature: u8 = match &header.kind {
            HeaderFieldKind::Path => b'o',
            HeaderFieldKind::BodySignature => b'g',
            _ => b's', // rest all types are encoded as strings
        };

        let signature_length = 1; // signature length is always u8 not u32, and for all our headers, it is going to be 1

        // header preamble
        temp.extend_from_slice(&[header_kind, signature_length, header_signature, 0]);
        // header_length_ctr += 4; // this is fixed for all

        let header_value_length = header.value.len() as u32;

        // add header value length
        match &header.kind {
            HeaderFieldKind::BodySignature => {
                temp.push(header_value_length as u8);
                // header_length_ctr += 1;
            }
            _ => {
                temp.extend_from_slice(&header_value_length.to_le_bytes());
                // header_length_ctr += 4;
            }
        }

        temp.extend_from_slice(header.value.as_bytes());
        // header_length_ctr += header.value.len();

        temp.push(0); // null terminator
                      // header_length_ctr += 1;

        ret.append(&mut temp);
    }

    ret
}

impl Message {
    pub fn serialize(mut self) -> Vec<u8> {
        let mtype = match self.kind {
            MessageKind::MethodCall => 1,
            MessageKind::MethodReply => 2,
        };

        // Endianness , message type, flags, dbus spec version
        let mut message = vec![b'l', mtype, 0, 1];

        // set body length
        message.extend_from_slice(&(self.body.len() as u32).to_le_bytes());

        // set id
        message.extend_from_slice(&self.id.to_le_bytes());

        let serialized_headers = serialize_headers(&self.headers);

        // header length -  to be calculated without padding
        message.extend_from_slice(&(serialized_headers.len() as u32).to_le_bytes());
        message.extend_from_slice(&serialized_headers);

        let required_padding = (8 - (message.len() % 8)) % 8;

        // padding to 8 byte boundary
        for _ in 0..required_padding {
            message.push(0);
        }

        // body
        message.append(&mut self.body);

        // no padding after body

        message
    }
}
