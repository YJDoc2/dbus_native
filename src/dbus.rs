use std::io::{IoSlice, IoSliceMut};

use nix::sys::socket;

use crate::message::*;
use crate::proxy::Proxy;

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

        let reply: Vec<u8> = buf.iter().filter(|v| **v != 0).map(|v| *v).collect();
        let reply = unsafe { String::from_utf8_unchecked(reply) };
        if !reply.starts_with("OK") {
            panic!("Authentication failed, got message : {}", reply);
        }
        socket::send(
            self.socket,
            "BEGIN\r\n".as_bytes(),
            socket::MsgFlags::empty(),
        )?;
        self.method_call(
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "org.freedesktop.DBus",
            "Hello",
        );

        Ok(())
    }

    fn method_call(&mut self, path: &str, dest: &str, interface: &str, member: &str) {
        let mut headers = Vec::with_capacity(4);

        headers.push(Header {
            kind: HeaderFieldKind::Path,
            value: path.to_string(),
        });
        headers.push(Header {
            kind: HeaderFieldKind::Destination,
            value: dest.to_string(),
        });
        headers.push(Header {
            kind: HeaderFieldKind::Interface,
            value: interface.to_string(),
        });
        headers.push(Header {
            kind: HeaderFieldKind::Member,
            value: member.to_string(),
        });

        let message = Message {
            kind: MessageKind::MethodCall,
            id: self.get_msg_id(),
            headers,
            body: vec![],
        };

        let serialized = message.serialize();
        // println!("{:?}", serialized);
        socket::sendmsg::<()>(
            self.socket,
            &[IoSlice::new(&serialized)],
            &[],
            socket::MsgFlags::empty(),
            None,
        )
        .unwrap();
        let mut reply = [0_u8; 270];
        let reply_buffer = IoSliceMut::new(&mut reply[0..]);
        let reply_rcvd = socket::recvmsg::<()>(
            self.socket,
            &mut [reply_buffer],
            None,
            socket::MsgFlags::empty(),
        )
        .unwrap();
        println!("{:?}", reply_rcvd);
        println!("{:?}", reply);
    }

    fn get_msg_id(&mut self) -> u32 {
        self.msg_ctr += 1;
        self.msg_ctr
    }

    pub fn proxy(&mut self, destination: String, path: String) -> Proxy {
        Proxy::new(self, destination, path)
    }
}
