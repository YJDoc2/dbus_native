// see https://dbus.freedesktop.org/doc/dbus-specification.html and
// https://dbus.freedesktop.org/doc/api/html/structDBusHeader.html

use crate::utils::adjust_padding;

pub enum Endian {
    Little,
    Big, // we do not support this unless explicitly requested in youki's issues
}

impl Endian {
    fn to_byte(&self) -> u8 {
        match self {
            Self::Big => b'b',
            Self::Little => b'l',
        }
    }
}

// NOTE that we do not support all of the possible values and options, only those
// which are relevant and used by youki
pub enum MessageType {
    MethodCall,
    MethodReturn,
    Error,
    Signal, // we will ignore this for all intents and purposes
}

pub enum HeaderFieldKind {
    Path,
    Interface,
    Member,
    ErrorName,
    ReplySerial,
    Destination,
    Sender,
    BodySignature,
    UnixFd, // we will not use this, just for the sake of completion
}

pub struct Header {
    pub kind: HeaderFieldKind,
    pub value: String,
}

pub struct Preamble {
    endian: Endian,
    mtype: MessageType,
    flags: u8,
    version: u8,
}

impl Preamble {
    fn new(mtype: MessageType) -> Self {
        Self {
            endian: Endian::Little,
            mtype,
            flags: 0,   // until we need some flags to be used, this is fixed
            version: 1, // this is fixed until dbus releases a new major version
        }
    }
}

pub struct Message {
    preamble: Preamble,
    serial: u32,
    headers: Vec<Header>,
    body: Vec<u8>,
}

impl Message {
    pub fn new(mtype: MessageType, serial: u32, headers: Vec<Header>, body: Vec<u8>) -> Self {
        let preamble = Preamble::new(mtype);
        Self {
            preamble,
            serial: serial,
            headers,
            body,
        }
    }
}

// serialize without padding
fn serialize_headers(headers: &[Header]) -> Vec<u8> {
    let mut ret = vec![];
    for header in headers {
        let mut temp = vec![];
        adjust_padding(&mut ret, 8);

        let header_kind: u8 = match &header.kind {
            HeaderFieldKind::Path => 1,
            HeaderFieldKind::Interface => 2,
            HeaderFieldKind::Member => 3,
            HeaderFieldKind::ErrorName => 4,
            HeaderFieldKind::ReplySerial => 5,
            HeaderFieldKind::Destination => 6,
            HeaderFieldKind::Sender => 7,
            HeaderFieldKind::BodySignature => 8,
            HeaderFieldKind::UnixFd => 9,
        };

        let header_signature: u8 = match &header.kind {
            HeaderFieldKind::Path => b'o',
            HeaderFieldKind::BodySignature => b'g',
            _ => b's', // rest all types are encoded as strings
        };

        let signature_length = 1; // signature length is always u8 not u32, and for all our headers, it is going to be 1

        // header preamble
        temp.extend_from_slice(&[header_kind, signature_length, header_signature, 0]);

        let header_value_length = header.value.len() as u32;

        // add header value length
        match &header.kind {
            HeaderFieldKind::BodySignature => {
                temp.push(header_value_length as u8);
            }
            _ => {
                temp.extend_from_slice(&header_value_length.to_le_bytes());
            }
        }

        temp.extend_from_slice(header.value.as_bytes());

        temp.push(0); // null terminator

        ret.append(&mut temp);
    }

    ret
}

impl Message {
    pub fn serialize(mut self) -> Vec<u8> {
        let mtype = match self.preamble.mtype {
            MessageType::MethodCall => 1,
            MessageType::MethodReturn => 2,
            MessageType::Error => 3,
            MessageType::Signal => 4,
        };

        // Endian, message type, flags, dbus spec version
        let mut message = vec![
            self.preamble.endian.to_byte(),
            mtype,
            self.preamble.flags,
            self.preamble.version,
        ];

        // set body length
        message.extend_from_slice(&(self.body.len() as u32).to_le_bytes());

        // set id
        message.extend_from_slice(&self.serial.to_le_bytes());

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
