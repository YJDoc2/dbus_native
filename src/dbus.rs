use std::io::{IoSlice, IoSliceMut};

use nix::sys::socket;

use crate::message::*;
use crate::proxy::Proxy;

const REPLY_BUF_SIZE: usize = 128; // seems good enough  tradeoff between extra size and repeated calls
pub struct DbusConnection {
    socket: i32,
    msg_ctr: u32,
    uid: u32,
}

fn uid_to_hex_str(uid: u32) -> String {
    let temp: Vec<_> = uid
        .to_string()
        .chars()
        .map(|c| format!("{:x}", c as u8))
        .collect();
    temp.join("")
}

impl DbusConnection {
    pub fn new(addr: &str, uid: u32) -> Result<Self, nix::Error> {
        let socket = socket::socket(
            socket::AddressFamily::Unix,
            socket::SockType::Stream,
            socket::SockFlag::empty(),
            None,
        )?;

        let addr = socket::UnixAddr::new(addr)?;
        socket::connect(socket, &addr)?;
        Ok(Self {
            socket,
            uid,
            msg_ctr: 0,
        })
    }

    pub fn authenticate(&mut self) -> Result<(), nix::Error> {
        let mut buf = [0; 64];
        socket::send(self.socket, &[b'\0'], socket::MsgFlags::empty())?;

        let msg = format!("AUTH EXTERNAL {}\r\n", uid_to_hex_str(self.uid));

        socket::send(self.socket, msg.as_bytes(), socket::MsgFlags::empty())?;

        socket::recv(self.socket, &mut buf, socket::MsgFlags::empty())?;

        let reply: Vec<u8> = buf.iter().filter(|v| **v != 0).copied().collect();
        let reply = unsafe { String::from_utf8_unchecked(reply) };
        if !reply.starts_with("OK") {
            panic!("Authentication failed, got message : {}", reply);
        }
        socket::send(
            self.socket,
            "BEGIN\r\n".as_bytes(),
            socket::MsgFlags::empty(),
        )?;

        let headers = vec![
            Header {
                kind: HeaderFieldKind::Path,
                value: HeaderValue::String("/org/freedesktop/DBus".to_string()),
            },
            Header {
                kind: HeaderFieldKind::Destination,
                value: HeaderValue::String("org.freedesktop.DBus".to_string()),
            },
            Header {
                kind: HeaderFieldKind::Interface,
                value: HeaderValue::String("org.freedesktop.DBus".to_string()),
            },
            Header {
                kind: HeaderFieldKind::Member,
                value: HeaderValue::String("Hello".to_string()),
            },
        ];
        self.send_message(MessageType::MethodCall, headers, vec![]);

        Ok(())
    }

    fn receive_complete_response(&mut self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(512);
        loop {
            let mut reply: [u8; REPLY_BUF_SIZE] = [0_u8; REPLY_BUF_SIZE];
            let reply_buffer = IoSliceMut::new(&mut reply[0..]);
            let reply_rcvd = socket::recvmsg::<()>(
                self.socket,
                &mut [reply_buffer],
                None,
                socket::MsgFlags::empty(),
            )
            .unwrap();
            let received_byte_count = reply_rcvd.bytes;

            ret.extend_from_slice(&reply[0..received_byte_count]);
            if received_byte_count < REPLY_BUF_SIZE {
                // if received byte count is less than buffer size, then we got all
                break;
            }
        }
        ret
    }

    pub fn send_message(
        &mut self,
        mtype: MessageType,
        headers: Vec<Header>,
        body: Vec<u8>,
    ) -> Vec<Message> {
        let message = Message::new(mtype, self.get_msg_id(), headers, body);
        let serialized = message.serialize();
        socket::sendmsg::<()>(
            self.socket,
            &[IoSlice::new(&serialized)],
            &[],
            socket::MsgFlags::empty(),
            None,
        )
        .unwrap();
        let reply = self.receive_complete_response();
        let mut ret = Vec::new();
        let mut buf = &reply[..];
        while !buf.is_empty() {
            let mut ctr = 0;
            let msg = Message::deserialize(&buf[ctr..], &mut ctr);
            buf = &buf[ctr..];
            ret.push(msg);
        }
        ret
    }

    fn get_msg_id(&mut self) -> u32 {
        self.msg_ctr += 1;
        self.msg_ctr
    }

    pub fn proxy(&mut self, destination: String, path: String) -> Proxy {
        Proxy::new(self, destination, path)
    }
}
