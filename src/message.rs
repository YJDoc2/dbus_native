// see https://dbus.freedesktop.org/doc/dbus-specification.html and
// https://dbus.freedesktop.org/doc/api/html/structDBusHeader.html

use crate::utils::{adjust_padding, align_counter};

#[derive(Debug)]
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
    fn from_byte(byte: u8) -> Self {
        match byte {
            b'l' => Self::Little,
            b'b' => Self::Big,
            _ => panic!("invalid endian {}", byte),
        }
    }
}

// NOTE that we do not support all of the possible values and options, only those
// which are relevant and used by youki
#[derive(Debug)]
pub enum MessageType {
    MethodCall,
    MethodReturn,
    Error,
    Signal, // we will ignore this for all intents and purposes
}

#[derive(Debug)]
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

impl HeaderFieldKind {
    fn signature(&self) -> u8 {
        match &self {
            Self::Path => b'o',
            Self::ReplySerial => b'u',
            Self::BodySignature => b'g',
            Self::UnixFd => b'u',
            _ => b's', // rest all are encoded as string
        }
    }
}

#[derive(Debug)]
pub enum HeaderValue {
    String(String),
    U32(u32),
}

impl HeaderValue {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::String(s) => s.as_bytes().into(),
            Self::U32(v) => v.to_le_bytes().into(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::String(s) => s.len(),
            Self::U32(v) => 4, // u32 is encoded as 4 bytes
        }
    }
}

#[derive(Debug)]
pub struct Header {
    pub kind: HeaderFieldKind,
    pub value: HeaderValue,
}

impl Header {
    fn parse(buf: &[u8], ctr: &mut usize) -> Self {
        let header_kind = match buf[*ctr] {
            1 => HeaderFieldKind::Path,
            2 => HeaderFieldKind::Interface,
            3 => HeaderFieldKind::Member,
            4 => HeaderFieldKind::ErrorName,
            5 => HeaderFieldKind::ReplySerial,
            6 => HeaderFieldKind::Destination,
            7 => HeaderFieldKind::Sender,
            8 => HeaderFieldKind::BodySignature,
            9 => HeaderFieldKind::UnixFd,
            _ => panic!("invalid header kind"),
        };

        *ctr += 1;

        let signature_length = buf[*ctr] as usize;
        *ctr += 1;

        //we ignore this, as we always parse the header
        let actual_signature =
            String::from_utf8(buf[*ctr..*ctr + signature_length].into()).unwrap();

        *ctr += signature_length;

        let expected_header_signature = header_kind.signature();

        let expected_signature = String::from_utf8([expected_header_signature].into()).unwrap();

        if actual_signature != expected_signature {
            panic!(
                "header signature mismatch, expected {}, found {}",
                expected_signature, actual_signature
            );
        }

        *ctr += 1; // accounting for extra null byte that is always there

        let value = match expected_header_signature {
            b'u' => {
                let ret =
                    HeaderValue::U32(u32::from_le_bytes(buf[*ctr..*ctr + 4].try_into().unwrap()));
                *ctr += 4;
                ret
            }
            b'o' => {
                let len = u32::from_le_bytes(buf[*ctr..*ctr + 4].try_into().unwrap()) as usize;
                *ctr += 4;
                let string = String::from_utf8(buf[*ctr..*ctr + len].into()).unwrap();
                *ctr += len;
                HeaderValue::String(string)
            }
            b's' => {
                let len = u32::from_le_bytes(buf[*ctr..*ctr + 4].try_into().unwrap()) as usize;
                *ctr += 4;
                let string = String::from_utf8(buf[*ctr..*ctr + len].into()).unwrap();
                *ctr += len;
                HeaderValue::String(string)
            }
            b'g' => {
                let len = buf[*ctr] as usize;
                *ctr += 1;
                let signature = String::from_utf8(buf[*ctr..*ctr + len].into()).unwrap();
                *ctr += len;
                *ctr += 1; // trailing null byte
                HeaderValue::String(signature)
            }
            _ => panic!("unexpected header signature {}", expected_header_signature),
        };
        Self {
            kind: header_kind,
            value,
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

        let header_signature: u8 = header.kind.signature();

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

        temp.extend_from_slice(&header.value.as_bytes());

        temp.push(0); // null terminator

        ret.append(&mut temp);
    }

    ret
}

fn deserialize_headers(buf: &[u8]) -> Vec<Header> {
    let mut ret = Vec::new();

    let mut ctr = 0;

    while ctr < buf.len() {
        align_counter(&mut ctr, 8);
        let header = Header::parse(buf, &mut ctr);
        ret.push(header);
        // todo move header parse in separate stuff
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

    pub fn deserialize(buf: Vec<u8>) -> Self {
        let mut counter = 0;

        let endian = Endian::from_byte(buf[0]);

        if !matches!(endian, Endian::Big) {
            panic!("we do not support big endian yet");
        }

        let mtype = match buf[1] {
            1 => MessageType::MethodCall,
            2 => MessageType::MethodReturn,
            3 => MessageType::Error,
            4 => MessageType::Signal,
            _ => panic!("invalid message type {}", buf[1]),
        };
        let _flags = buf[2]; // we basically ignore flags
        let version = buf[3];
        if version != 1 {
            panic!("when did dbus release new version?!?!?!");
        }
        counter += 4;

        let preamble = Preamble::new(mtype);

        let body_length =
            u32::from_le_bytes(buf[counter..counter + 4].try_into().unwrap()) as usize;
        counter += 4;

        let serial = u32::from_le_bytes(buf[counter..counter + 4].try_into().unwrap());
        counter += 4;

        let header_array_length =
            u32::from_le_bytes(buf[counter..counter + 4].try_into().unwrap()) as usize;
        counter += 4;

        let headers = deserialize_headers(&buf[counter..counter + header_array_length]);

        counter += header_array_length;
        align_counter(&mut counter, 8);

        let body = Vec::from(&buf[counter..counter + body_length]);

        Self {
            preamble,
            serial,
            headers,
            body,
        }
    }
}
