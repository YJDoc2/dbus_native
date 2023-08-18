use crate::serialize::Variant;

mod containers;
mod dbus;
mod message;
mod proxy;
mod serialize;
mod utils;
fn main() {
    let mut dbus = dbus::DbusConnection::new("/run/user/1000/bus", 1000).unwrap();
    dbus.authenticate().unwrap();
    let mut proxy = dbus.proxy(
        "org.freedesktop.DBus".to_string(),
        "/org/freedesktop/DBus".to_string(),
    );
    let reply = proxy.method_call::<(), String>("org.freedesktop.DBus", "GetId", None);
    println!("{:?}", reply);

    let mut proxy = dbus.proxy(
        "org.freedesktop.systemd1".to_string(),
        "/org/freedesktop/systemd1".to_string(),
    );
    let body = (
        "org.freedesktop.systemd1.Manager".to_string(),
        "Version".to_string(),
    );
    let reply = proxy.method_call::<_, Variant<String>>(
        "org.freedesktop.DBus.Properties",
        "Get",
        Some(body),
    );
    println!("{:?}", reply);

    let body = (
        "org.freedesktop.systemd1.Manager".to_string(),
        "ControlGroup".to_string(),
    );
    let reply = proxy.method_call::<_, Variant<String>>(
        "org.freedesktop.DBus.Properties",
        "Get",
        Some(body),
    );
    println!("{:?}", reply);
}
