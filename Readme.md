# Dbus Native

---


This is repo for initial development and testing for Rust-native bindings for Dbus communication.

Note that this does not provide full dbus functionalities, nor intends to, only the stuff needed by youki. However, you might find this useful as a reference if you want to write your own bindings. This does not have any external dependencies apart from `nix` crate for sockets.

See [this](https://github.com/containers/youki/issues/2208) for background on why we decided to write custom bindings and not use existing libraries.


Note that once this is merged in youki, this repo may not be kept up-to-date, and changes might be done only to youki. This serves as a convenient testing space for developing.