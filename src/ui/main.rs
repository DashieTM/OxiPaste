use adw::{prelude::AdwApplicationWindowExt, Application, ApplicationWindow};
use dbus::blocking::Connection;
use glib::clone;
use gtk::{
    gdk::Key,
    gdk_pixbuf::Pixbuf,
    gio::{self, ActionEntry, MemoryInputStream},
    glib::{Cast, Propagation},
    prelude::{
        ActionMapExtManual, ApplicationExt, ApplicationExtManual, BoxExt, ButtonExt, GtkWindowExt,
        WidgetExt,
    },
    Button, Image, Orientation, PolicyType, ScrolledWindow,
};

use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::{env, rc::Rc, time::Duration};

pub fn main() -> adw::glib::ExitCode {
    let mut css_string = "".to_string();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mut argiter = args.iter();
        argiter.next().unwrap();
        match argiter.next().unwrap().as_str() {
            "--css" => {
                let next = argiter.next();
                if next.is_some() {
                    css_string = next.unwrap().clone();
                }
            }
            _ => {
                print!(
                    "usage:
    --css: use a specific path to load a css style sheet.
    --help: show this message.\n"
                );
                return adw::glib::ExitCode::FAILURE;
            }
        }
    } else {
        let config_dir = oxilib::create_config_folder("oxipaste");
        css_string = oxilib::create_css(
            &config_dir,
            "style.css",
            r#".main-window {
            opacity: 100%;
            border-radius: 10px;
        "#,
        )
        .to_str()
        .expect("Could not process css file.")
        .into();
    }
    let app = Application::builder()
        .application_id("org.Xetibo.OxiPaste")
        .build();
    app.connect_startup(move |_| {
        if !gtk::is_initialized() {
            adw::init().unwrap();
        }
        load_css(&css_string);
    });

    app.connect_activate(run_ui);
    app.run_with_args(&[""])
}

fn load_css(css_string: &String) {
    let context_provider = gtk::CssProvider::new();
    if !css_string.is_empty() {
        context_provider.load_from_path(css_string);
    }

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &context_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn run_ui(app: &Application) {
    let window = Rc::new(ApplicationWindow::new(app));
    let window_ref = window.clone();
    let window_ref2 = window.clone();
    let main_box = gtk::Box::new(Orientation::Vertical, 5);
    let all_items_box = gtk::Box::new(Orientation::Vertical, 5);
    let button_box = gtk::Box::new(Orientation::Horizontal, 5);
    let clear_button = Button::new();
    let items = get_items();

    clear_button.connect_clicked(move |button| {
        // (*item_ref).clear();
        button
            .activate_action("win.delete_all", None)
            .expect("Could not delete all entries.");
        button
            .activate_action("win.close", None)
            .expect("Could not close the application.");
    });
    clear_button.set_label("clear clipboard");
    clear_button.set_margin_start(5);
    clear_button.set_margin_end(5);
    clear_button.set_margin_top(5);
    clear_button.set_margin_bottom(5);

    let delete_all_action = ActionEntry::builder("close")
        .activate(move |window: &ApplicationWindow, _, _| {
            delete_all();
            let boxy = window
                .first_child()
                .unwrap()
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Box>()
                .expect("Could not cast to gtk box.");
            loop {
                let child = boxy.first_child();
                if child.is_none() {
                    break;
                }
                boxy.remove(&child.unwrap());
            }
        })
        .build();
    let close_window = ActionEntry::builder("delete_all")
        .activate(move |window: &ApplicationWindow, _, _| {
            window.close();
        })
        .build();
    window.add_action_entries([delete_all_action, close_window]);

    button_box.append(&clear_button);
    main_box.append(&button_box);

    for (iter, (data, mimetype)) in items.into_iter().enumerate() {
        let loop_ref = window_ref.clone();
        all_items_box.append(&item(loop_ref, iter, data, mimetype));
    }

    let key_event_controller = gtk::EventControllerKey::new();
    key_event_controller.connect_key_pressed(move |_controller, key, _keycode, _state| match key {
        Key::Escape => {
            window_ref2.close();
            Propagation::Stop
        }
        _ => Propagation::Proceed,
    });
    window.add_controller(key_event_controller);
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_width_request(600);
    scrolled_window.set_height_request(400);
    scrolled_window.set_hexpand(false);
    scrolled_window.set_hscrollbar_policy(PolicyType::Never);
    scrolled_window.add_css_class("item-window");
    main_box.append(&scrolled_window);
    scrolled_window.set_child(Some(&all_items_box));
    window.set_content(Some(&main_box));
    window.init_layer_shell();
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_layer(Layer::Overlay);
    window.add_css_class("main-window");

    let first = all_items_box.first_child();
    if first.is_some() {
        let focus = first.unwrap().first_child().unwrap();
        focus.grab_focus();
    }

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
    type ResultType = Result<(Vec<(Vec<u8>, String)>,), dbus::Error>;
    let conn = Connection::new_session().unwrap();
    let proxy = conn.with_proxy(
        "org.Xetibo.OxiPasteDaemon",
        "/org/Xetibo/OxiPasteDaemon",
        Duration::from_millis(1000),
    );
    let res: ResultType = proxy.method_call("org.Xetibo.OxiPasteDaemon", "GetAll", ());
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

fn item(loop_ref: Rc<ApplicationWindow>, iter: usize, data: Vec<u8>, mimetype: String) -> gtk::Box {
    let key_ref = loop_ref.clone();
    let item_box = gtk::Box::new(Orientation::Horizontal, 10);
    item_box.add_css_class("item-box");

    let key_event_controller = gtk::EventControllerKey::new();
    key_event_controller.connect_key_pressed(
        clone!(@strong item_box => move |_controller, key, _keycode, _state| match key {
            Key::Delete => {
                item_box.unparent();
                Propagation::Stop
            }
            Key::ISO_Enter => {
                key_ref.hide();
                copy_to_clipboard(iter);
                key_ref.close();
                Propagation::Stop
            }
            // Key::Tab => {
            //     Propagation::Proceed
            // }
            _ => Propagation::Proceed,
        }),
    );
    item_box.add_controller(key_event_controller);
    let selection_button = Button::new();
    selection_button.add_css_class("item-button");
    let mut image = Image::new();
    image.add_css_class("image");
    if mimetype.contains("image") {
        image.set_height_request(300);
        set_image(data, &mut image);
        // image.add_controller(gesture_click);
        selection_button.set_child(Some(&image));
        selection_button.set_hexpand(true);
        selection_button.set_height_request(300);
        selection_button.set_height_request(300);
    } else if mimetype.contains("text") {
        selection_button.set_label(&String::from_utf8_lossy(&data));
        selection_button.set_hexpand(true);
    }
    item_box.append(&selection_button);

    selection_button.connect_clicked(move |_| {
        loop_ref.hide();
        copy_to_clipboard(iter);
        loop_ref.close();
    });

    let delete_button = Button::new();
    delete_button.add_css_class("delete-button");
    delete_button.set_can_focus(false);
    delete_button.set_label("x");
    delete_button.set_vexpand(false);
    delete_button.set_valign(gtk::Align::Start);
    delete_button.connect_clicked(clone!(@strong item_box => move |_| {
        delete_index(iter);
        item_box.unparent();
    }));
    item_box.append(&delete_button);

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
