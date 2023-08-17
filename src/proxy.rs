use crate::serialize::DbusSerialize;

use super::dbus::DbusConnection;
use super::message::*;

pub struct Proxy<'conn> {
    conn: &'conn mut DbusConnection,
    dest: String,
    path: String,
}

impl<'conn> Proxy<'conn> {
    pub fn new(conn: &'conn mut DbusConnection, dest: String, path: String) -> Self {
        Self { conn, dest, path }
    }

    pub fn method_call<T: DbusSerialize>(
        &mut self,
        interface: &str,
        member: &str,
        body: Option<T>,
    ) -> Vec<Message> {
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
                value: HeaderValue::String(T::get_signature()),
            });
            v.serialize(&mut serialized_body);
        }
        self.conn
            .send_message(MessageType::MethodCall, headers, serialized_body)
    }
}
