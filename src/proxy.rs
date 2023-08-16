use super::dbus::DbusConnection;

pub struct Proxy<'conn> {
    conn: &'conn mut DbusConnection,
    dest: String,
    path: String,
}

impl<'conn> Proxy<'conn> {
    pub fn new(conn: &'conn mut DbusConnection, dest: String, path: String) -> Self {
        Self { conn, dest, path }
    }
}
