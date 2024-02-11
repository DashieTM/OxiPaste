pub mod item;

use adw::{prelude::AdwWindowExt, Application, Window};
use dbus::{arg::RefArg, blocking::Connection};
use gtk::{
    prelude::{
        ApplicationExt, ApplicationExtManual, BoxExt, ButtonExt, GtkApplicationExt, GtkWindowExt,
        WidgetExt,
    },
    Button, GestureClick, Label, Orientation,
};
use gtk4_layer_shell::{Edge, LayerShell};
use std::{process::Command, rc::Rc, time::Duration};
use wl_clipboard_rs::copy::{MimeType, Options, ServeRequests, Source};

pub fn main() {
    let app = Application::builder()
        .application_id("org.Xetibo.OxiPaste")
        .build();
    app.connect_startup(move |_| {
        if !gtk::is_initialized() {
            adw::init().unwrap();
        }
        // load_css(&css_string);
    });

    app.connect_activate(run_ui);
    app.run();
}

fn run_ui(app: &Application) {
    let window = Rc::new(Window::new());
    let window_ref = window.clone();
    app.add_window(&*window);
    let all_items_box = gtk::Box::new(Orientation::Vertical, 5);
    let items = get_items();
    for (iter, (data, _)) in items.into_iter().enumerate() {
        let loop_ref = window_ref.clone();
        // TODO handle images etc
        let item_box = gtk::Box::new(Orientation::Vertical, 5);
        let gesture_click = GestureClick::new();
        let text = Label::new(Some(&String::from_utf8_lossy(&data)));
        item_box.append(&text);
        gesture_click.connect_pressed(move |_, _, _, _| {
            loop_ref.hide();
            copy_to_clipboard(iter); //.expect("wat");
            loop_ref.close();
        });
        item_box.add_controller(gesture_click);
        all_items_box.append(&item_box);
    }

    window.set_content(Some(&all_items_box));
    window.init_layer_shell();
    // window.set_layer()

    window.present();
}

fn copy_to_clipboard(index: usize) -> bool {
    let conn = Connection::new_session().unwrap();
    let proxy = conn.with_proxy(
        "org.Xetibo.OxiPasteDaemon",
        "/org/Xetibo/OxiPasteDaemon",
        Duration::from_millis(1000),
    );
    let res: Result<(bool,), dbus::Error> =
        proxy.method_call("org.Xetibo.OxiPasteDaemon", "Paste", (index as u32,));
    if res.is_err() {
        return false;
    }
    res.unwrap().0
}

fn get_items() -> Vec<(Vec<u8>, String)> {
    let conn = Connection::new_session().unwrap();
    let proxy = conn.with_proxy(
        "org.Xetibo.OxiPasteDaemon",
        "/org/Xetibo/OxiPasteDaemon",
        Duration::from_millis(1000),
    );
    let res: Result<(Vec<(Vec<u8>, String)>,), dbus::Error> =
        proxy.method_call("org.Xetibo.OxiPasteDaemon", "GetAll", ());
    if res.is_err() {
        return Vec::new();
    }
    res.unwrap().0
}

fn delete_all() -> bool {
    let conn = Connection::new_session().unwrap();
    let proxy = conn.with_proxy(
        "org.Xetibo.OxiPasteDaemon",
        "/org/Xetibo/OxiPasteDaemon",
        Duration::from_millis(1000),
    );
    let res: Result<(bool,), dbus::Error> =
        proxy.method_call("org.Xetibo.OxiPasteDaemon", "DeleteAll", ());
    if res.is_err() {
        return false;
    }
    res.unwrap().0
}

fn delete_index(index: usize) -> bool {
    let conn = Connection::new_session().unwrap();
    let proxy = conn.with_proxy(
        "org.Xetibo.OxiPasteDaemon",
        "/org/Xetibo/OxiPasteDaemon",
        Duration::from_millis(1000),
    );
    let res: Result<(bool,), dbus::Error> = proxy.method_call(
        "org.Xetibo.OxiPasteDaemon",
        "DeleteAtIndex",
        (index as u32,),
    );
    if res.is_err() {
        return false;
    }
    res.unwrap().0
}
