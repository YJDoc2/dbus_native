mod dbus;
mod message;
mod proxy;
mod serialize;
mod containers;
mod utils;
fn main() {
    let mut dbus = dbus::DbusConnection::new("/run/user/1000/bus", 1000).unwrap();
    dbus.authenticate().unwrap();
}
