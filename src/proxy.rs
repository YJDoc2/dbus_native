use crate::serialize::DbusSerialize;

use super::dbus::DbusConnection;
use super::message::*;

#[derive(Debug)]
pub struct DbusError(String);

pub type Result<T> = std::result::Result<T, DbusError>;

pub struct Proxy<'conn> {
    conn: &'conn mut DbusConnection,
    dest: String,
    path: String,
}

impl<'conn> Proxy<'conn> {
    pub fn new(conn: &'conn mut DbusConnection, dest: String, path: String) -> Self {
        Self { conn, dest, path }
    }

    pub fn method_call<Body: DbusSerialize, Output: DbusSerialize>(
        &mut self,
        interface: &str,
        member: &str,
        body: Option<Body>,
    ) -> Result<Output> {
        let mut headers = Vec::with_capacity(4);

        headers.push(Header {
            kind: HeaderFieldKind::Path,
            value: HeaderValue::String(self.path.clone()),
        });
        headers.push(Header {
            kind: HeaderFieldKind::Destination,
            value: HeaderValue::String(self.dest.clone()),
        });
        headers.push(Header {
            kind: HeaderFieldKind::Interface,
            value: HeaderValue::String(interface.to_string()),
        });
        headers.push(Header {
            kind: HeaderFieldKind::Member,
            value: HeaderValue::String(member.to_string()),
        });

        let mut serialized_body = vec![];
        if let Some(v) = body {
            headers.push(Header {
                kind: HeaderFieldKind::BodySignature,
                value: HeaderValue::String(Body::get_signature()),
            });
            v.serialize(&mut serialized_body);
        }
        let reply_messages =
            self.conn
                .send_message(MessageType::MethodCall, headers, serialized_body);

        let error_message: Vec<_> = reply_messages
            .iter()
            .filter(|m| m.preamble.mtype == MessageType::Error)
            .collect();

        // if any error, return error
        if !error_message.is_empty() {
            let msg = error_message[0];
            if msg.body.is_empty() {
                return Err(DbusError("Unknown Dbus Error".to_owned()));
            } else {
                let mut ctr = 0;
                return Err(DbusError(String::deserialize(&msg.body, &mut ctr)));
            }
        }

        let reply: Vec<_> = reply_messages
            .iter()
            .filter(|m| m.preamble.mtype == MessageType::MethodReturn)
            .collect();

        // we are only going to consider first reply, cause... so.
        let reply = reply[0];

        let headers = &reply.headers;
        let expected_signature = Output::get_signature();
        let signature_header: Vec<_> = headers
            .iter()
            .filter(|h| h.kind == HeaderFieldKind::BodySignature)
            .collect();
        if signature_header.is_empty() && !reply.body.is_empty() {
            return Err(DbusError(
                "Body non empty, but body signature header missing".to_string(),
            ));
        }

        if expected_signature == *"" {
            // fixme  . This hack is for () type, when nno reply body is expected
            let mut ctr = 0;
            return Ok(Output::deserialize(&[], &mut ctr));
        }

        let actual_signature = match &signature_header[0].value {
            HeaderValue::String(s) => s,
            _ => unreachable!("body signature header will always be string type"),
        };

        if *actual_signature != expected_signature {
            return Err(DbusError(format!(
                "reply signature mismatch : expected {}, found {}",
                expected_signature, actual_signature
            )));
        }

        let mut ctr = 0;
        let ret = Output::deserialize(&reply.body, &mut ctr);
        Ok(ret)
    }
}
