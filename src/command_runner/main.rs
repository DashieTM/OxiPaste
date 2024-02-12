use dbus::blocking::Connection;
use std::time::Duration;
pub fn main() {
    let conn = Connection::new_session().unwrap();
    let proxy = conn.with_proxy(
        "org.Xetibo.OxiPasteDaemon",
        "/org/Xetibo/OxiPasteDaemon",
        Duration::from_millis(1000),
    );
    let res: Result<(bool,), dbus::Error> =
        proxy.method_call("org.Xetibo.OxiPasteDaemon", "Copy", ());
    if res.is_err() {
        println!("Could not establish connection to OxiPaste dbus daemon.");
    }
}
