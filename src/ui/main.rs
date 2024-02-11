pub mod item;

use adw::{prelude::AdwWindowExt, Application, Window};
use dbus::{arg::RefArg, blocking::Connection};
use gtk::{
    builders::ImageBuilder,
    gdk_pixbuf::Pixbuf,
    gio::{self, MemoryInputStream},
    prelude::{
        ApplicationExt, ApplicationExtManual, BoxExt, ButtonExt, GtkApplicationExt, GtkWindowExt,
        WidgetExt,
    },
    Button, GestureClick, Image, Label, Orientation,
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
    for (iter, (data, mimetype)) in items.into_iter().enumerate() {
        let loop_ref = window_ref.clone();
        // TODO handle images etc
        all_items_box.append(&item(loop_ref, iter, data, mimetype));
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

fn item(loop_ref: Rc<Window>, iter: usize, data: Vec<u8>, mimetype: String) -> gtk::Box {
    let item_box = gtk::Box::new(Orientation::Vertical, 10);

    let gesture_click = GestureClick::new();
    gesture_click.connect_pressed(move |_, _, _, _| {
        loop_ref.hide();
        copy_to_clipboard(iter); //.expect("wat");
        loop_ref.close();
    });
    item_box.add_controller(gesture_click);
    if mimetype.contains("image") {
        // let mimetype = mimetype.trim_start_matches("image/");
        // match mimetype {
        //     "png" => {
        let mut image = Image::new();
        image.set_height_request(300);
        set_image(data, &mut image);
        item_box.append(&image);
        //     }
        //     _ => (),
        // }
    } else if mimetype.contains("text") {
        let text = Label::new(Some(&String::from_utf8_lossy(&data)));
        item_box.append(&text);
    }
    item_box
}

fn set_image(data: Vec<u8>, image: &mut Image) {
    let bytes = gtk::glib::Bytes::from(&data);
    if bytes.is_empty() {
        return;
    }
    let stream = MemoryInputStream::from_bytes(&bytes);
    let pixbuf = Pixbuf::from_stream(&stream, gio::Cancellable::NONE).unwrap();
    image.set_from_pixbuf(Some(&pixbuf));
}
