// see https://dbus.freedesktop.org/doc/dbus-specification.html and
// https://dbus.freedesktop.org/doc/api/html/structDBusHeader.html

use crate::utils::{adjust_padding, align_counter, DbusError, Result};

#[derive(Debug)]
/// Indicates the endian of message
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

/// Represents the type of header data
// there are others, but these are the only once we need
#[derive(Debug, PartialEq, Eq)]
pub enum HeaderSignature {
    Object,
    U32,
    String,
    Signature,
}

impl HeaderSignature {
    fn to_byte(&self) -> u8 {
        match self {
            Self::Object => b'o',
            Self::Signature => b'g',
            Self::String => b's',
            Self::U32 => b'u',
        }
    }
    fn from_byte(byte: u8) -> Self {
        match byte {
            b'o' => Self::Object,
            b'g' => Self::Signature,
            b's' => Self::String,
            b'u' => Self::U32,
            _ => panic!("unexpected signature {}", byte),
        }
    }
}

/// Type of message
#[derive(Debug, PartialEq, Eq)]
pub enum MessageType {
    MethodCall,
    MethodReturn,
    Error,
    Signal, // we will ignore this for all intents and purposes
}

/// Represents the kind of header
#[derive(Debug, PartialEq, Eq)]
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
    fn signature(&self) -> HeaderSignature {
        match &self {
            Self::Path => HeaderSignature::Object,
            Self::ReplySerial => HeaderSignature::U32,
            Self::BodySignature => HeaderSignature::Signature, // this is also encoded as string, but we need special handling for how its length is encoded
            Self::UnixFd => HeaderSignature::U32,
            _ => HeaderSignature::String, // rest all are encoded as string
        }
    }
}

// This is separated from header field kind, because I wanted HeaderFiledKind to be u8 like,
// directly comparable, passable thing
#[derive(Debug)]
pub enum HeaderFieldValue {
    String(String),
    U32(u32),
}

impl HeaderFieldValue {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::String(s) => {
                let mut t: Vec<u8> = s.as_bytes().into();
                t.push(0); // null byte terminator
                t
            }
            Self::U32(v) => v.to_le_bytes().into(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::String(s) => s.len(),
            Self::U32(_) => 4, // u32 is encoded as 4 bytes
        }
    }
}

#[derive(Debug)]
pub struct Header {
    pub kind: HeaderFieldKind,
    pub value: HeaderFieldValue,
}

impl Header {
    /// Parses a single header from given u8 vec,
    /// assuming the header to start from given counter
    fn parse(buf: &[u8], ctr: &mut usize) -> Result<Self> {
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
            _ => panic!("invalid header kind"), // should not occur unless we screw up parsing somewhere else
        };

        // account for the header_kind byte
        *ctr += 1;

        // length of signature is always <255
        let signature_length = buf[*ctr] as usize;
        *ctr += 1;

        // we only support string, u32 signature and object,
        // all of which have signature of 1 byte
        if signature_length != 1 {
            return Err(DbusError::IncompleteImplementation(format!(
                "some complex valued header is sent"
            )));
        }

        let actual_signature = HeaderSignature::from_byte(buf[*ctr]);

        // we can simply += 1, but I think this is more sensible
        *ctr += signature_length;

        let expected_signature = header_kind.signature();

        if actual_signature != expected_signature {
            return Err(DbusError::IncorrectMessage(format!(
                "header signature mismatch, expected {:?}, found {:?}",
                expected_signature, actual_signature
            )));
        }

        *ctr += 1; // accounting for extra null byte that is always there

        let value = match expected_signature {
            HeaderSignature::U32 => {
                let ret = HeaderFieldValue::U32(u32::from_le_bytes(
                    buf[*ctr..*ctr + 4].try_into().unwrap(), // we ca unwrap here as we know 4 byte buffer will satisfy [u8;4]
                ));
                *ctr += 4;
                ret
            }
            // both are encoded as string
            HeaderSignature::Object | HeaderSignature::String => {
                let len = u32::from_le_bytes(buf[*ctr..*ctr + 4].try_into().unwrap()) as usize;
                *ctr += 4;
                let string = String::from_utf8(buf[*ctr..*ctr + len].into()).unwrap();
                *ctr += len + 1; // +1 to account for null
                HeaderFieldValue::String(string)
            }
            // only difference here is that length is 1 byte, not 4 bytes
            HeaderSignature::Signature => {
                let len = buf[*ctr] as usize;
                *ctr += 1;
                let signature = String::from_utf8(buf[*ctr..*ctr + len].into()).unwrap();
                *ctr += len + 1; //+1 to account for null byte
                HeaderFieldValue::String(signature)
            }
        };
        Ok(Self {
            kind: header_kind,
            value,
        })
    }
}

/// Message preamble of initial 4 bytes
#[derive(Debug)]
pub struct Preamble {
    endian: Endian,
    pub mtype: MessageType,
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

/// Represents a complete message transported over dbus connection
#[derive(Debug)]
pub struct Message {
    /// Initial 4 byte preamble needed for all messages
    pub preamble: Preamble,
    /// Serial ID of message
    pub serial: u32,
    // Message headers
    pub headers: Vec<Header>,
    /// Actual body, serialized
    pub body: Vec<u8>,
}

impl Message {
    pub fn new(mtype: MessageType, serial: u32, headers: Vec<Header>, body: Vec<u8>) -> Self {
        let preamble = Preamble::new(mtype);
        Self {
            preamble,
            serial,
            headers,
            body,
        }
    }
}

// NOTE that this does not add padding after last header, because we need
// non-padded header length
// This alignment must be done  separately after this
fn serialize_headers(headers: &[Header]) -> Vec<u8> {
    let mut ret = vec![];

    for header in headers {
        // all headers are always 8 byte aligned
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

        let header_signature: u8 = header.kind.signature().to_byte();

        let signature_length = 1; // signature length is always u8 not u32, and for all our headers, it is going to be 1

        // header preamble
        ret.extend_from_slice(&[header_kind, signature_length, header_signature, 0]);

        let header_value_length = header.value.len() as u32;

        // add header value length
        match &header.kind {
            HeaderFieldKind::BodySignature => {
                // signature length is always 1 byte
                ret.push(header_value_length as u8);
            }
            HeaderFieldKind::ReplySerial | HeaderFieldKind::UnixFd => { /* do nothing */ }
            _ => {
                ret.extend_from_slice(&header_value_length.to_le_bytes());
            }
        }

        ret.extend_from_slice(&header.value.as_bytes());
    }

    ret
}

fn deserialize_headers(buf: &[u8]) -> Result<Vec<Header>> {
    let mut ret = Vec::new();

    let mut ctr = 0;
    // headers are always aligned at 8 byte boundary
    align_counter(&mut ctr, 8);
    while ctr < buf.len() {
        let header = Header::parse(buf, &mut ctr)?;
        align_counter(&mut ctr, 8);
        ret.push(header);
    }
    Ok(ret)
}

impl Message {
    /// Serialize the given message into u8 vec
    pub fn serialize(mut self) -> Vec<u8> {
        let mtype = match self.preamble.mtype {
            MessageType::MethodCall => 1,
            MessageType::MethodReturn => 2,
            MessageType::Error => 3,
            MessageType::Signal => 4,
        };

        // preamble
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

        adjust_padding(&mut message, 8);

        // body
        message.append(&mut self.body);

        // no padding after body

        message
    }

    pub fn deserialize(buf: &[u8], counter: &mut usize) -> Result<Self> {
        let endian = Endian::from_byte(buf[*counter]);

        if !matches!(endian, Endian::Little) {
            return Err(DbusError::IncompleteImplementation(
                "we do not support big endian yet".into(),
            ));
        }

        let mtype = match buf[*counter + 1] {
            1 => MessageType::MethodCall,
            2 => MessageType::MethodReturn,
            3 => MessageType::Error,
            4 => MessageType::Signal,
            _ => panic!("invalid message type {}", buf[*counter + 1]),
        };

        let _flags = buf[*counter + 2]; // we basically ignore flags
        let version = buf[*counter + 3];

        if version != 1 {
            panic!("when did dbus release new version?!?!?!");
        }

        *counter += 4; // account for preamble bytes

        let preamble = Preamble::new(mtype);

        let body_length =
            u32::from_le_bytes(buf[*counter..*counter + 4].try_into().unwrap()) as usize;
        *counter += 4;

        let serial = u32::from_le_bytes(buf[*counter..*counter + 4].try_into().unwrap());
        *counter += 4;

        let header_array_length =
            u32::from_le_bytes(buf[*counter..*counter + 4].try_into().unwrap()) as usize;
        *counter += 4;

        let headers = deserialize_headers(&buf[*counter..*counter + header_array_length])?;
        *counter += header_array_length;
        align_counter(counter, 8);

        // we do not deserialize body here, and istead let the caller do it as needed
        // that way we don't have do deal with error checking or validating the body signature etc
        let body = Vec::from(&buf[*counter..*counter + body_length]);
        *counter += body_length;

        Ok(Self {
            preamble,
            serial,
            headers,
            body,
        })
    }
}
