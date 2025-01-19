use config::{default_config, Config, ConfigOptional};
use iced::futures;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use wl_clipboard_rs::copy::{Options, Source};
use wl_clipboard_rs::paste::{get_contents, ClipboardType, Error, MimeType, Seat};

pub mod config;
pub mod dbus;
// TODO wip
pub mod protocol;

pub enum ReverseCommand {
    SendLatest((Vec<u8>, String)),
    SendAll(Vec<(Vec<u8>, String)>),
}

pub enum Command {
    WriteToFile,
    ShutDown,
    Copy,
    DeleteAtIndex(usize),
    DeleteAll,
    GetLatest,
    GetAll,
    Paste(usize),
    PasteAndDelete(usize),
}

static CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| oxilib::create_config_folder("oxipaste"));

static CONFIG: Lazy<Config> = Lazy::new(|| {
    oxilib::create_config::<Config, ConfigOptional>(&CONFIG_DIR, "config.toml", default_config())
});

fn main() {
    std::thread::spawn(|| {
        start_wl_copy_runner();
    });
    let (sender, receiver) = mpsc::channel::<Command>();
    let (reverse_sender, reverse_receiver) = mpsc::channel::<ReverseCommand>();
    std::thread::spawn(move || {
        let _ = futures::executor::block_on(dbus::run(sender, reverse_receiver));
    });
    let mut items: IndexMap<Vec<u8>, String> = get_items_from_file();
    let mut time = std::time::SystemTime::now();
    loop {
        let result = receiver.recv();
        let len = items.len();
        let new_time = std::time::SystemTime::now();

        if new_time
            .duration_since(time)
            .unwrap_or(Duration::from_millis(0))
            > Duration::from_secs_f64(300.0)
        {
            write_items_to_file(&items);
        }
        time = new_time;

        // clean memory in order to not leak
        // can later be configured with config file or something
        if len > CONFIG.max_items {
            let mut new_items = IndexMap::new();
            let iter = items.into_iter();
            for item in iter {
                new_items.insert(item.0, item.1);
            }
            items = new_items;
        }
        if let Ok(command) = result {
            match command {
                Command::WriteToFile => {
                    write_items_to_file(&items);
                }
                Command::ShutDown => {
                    write_items_to_file(&items);
                    break;
                }
                Command::Copy => get_items(&mut items),
                Command::DeleteAtIndex(index) => {
                    items.shift_remove_index(index);
                }
                Command::DeleteAll => {
                    items.clear();
                    clear_items_file();
                }
                Command::GetLatest => reverse_sender
                    .send(ReverseCommand::SendLatest(paste_latest(&mut items)))
                    .expect("Could not send command"),
                Command::GetAll => reverse_sender
                    .send(ReverseCommand::SendAll(items.clone().into_iter().collect()))
                    .expect("Could not send command"),
                Command::Paste(index) => copy_to_clipboard(&items, index),
                Command::PasteAndDelete(index) => {
                    copy_to_clipboard(&items, index);
                    items.shift_remove_index(index);
                }
            }
        }
    }
}

fn ensure_items_file() -> PathBuf {
    let item_file = CONFIG_DIR.join("items");
    if !item_file.is_file() {
        fs::File::create(&item_file).expect("Could not create item file.");
    }
    item_file
}

fn clear_items_file() {
    let item_file = ensure_items_file();
    let file = fs::File::options()
        .write(true)
        .append(false)
        .open(&item_file)
        .unwrap();
    file.set_len(0).expect("Could not set size to 0");
}

fn write_items_to_file(items: &IndexMap<Vec<u8>, String>) {
    let item_file = ensure_items_file();
    for item in items {
        let str_to_write_opt = String::from_utf8(item.0.to_vec());
        if let Ok(str_to_write) = str_to_write_opt {
            fs::write(&item_file, format!("{}<>:<>{}<><>\n", str_to_write, item.1))
                .expect("Could not write default css content.");
        }
    }
}

fn get_items_from_file() -> IndexMap<Vec<u8>, String> {
    let mut items = IndexMap::new();
    let item_file = ensure_items_file();
    let mut buffer = String::from("");
    let mut file = fs::File::open(&item_file).unwrap();
    file.read_to_string(&mut buffer)
        .expect("Could not read file");
    let lines: Vec<(&str, &str)> = buffer
        .split("<><>\n")
        .filter_map(|elem| elem.split_once("<>:<>"))
        .collect();
    for (key, value) in lines {
        items.insert(key.as_bytes().to_vec(), value.to_string());
    }
    items
}

fn copy_to_clipboard(items: &IndexMap<Vec<u8>, String>, index: usize) {
    let item = items.get_index(index);
    if item.is_none() {
        eprintln!("Tried to access index {} which is none", index);
        return;
    }
    let item = item.unwrap();

    let mut opts = Options::new();
    opts.trim_newline(true);
    opts.clipboard(wl_clipboard_rs::copy::ClipboardType::Regular);
    let res = opts.copy(
        Source::Bytes(item.0.clone().into()),
        match item.1.as_str() {
            "text/plain" => wl_clipboard_rs::copy::MimeType::Text,
            _ => wl_clipboard_rs::copy::MimeType::Specific(item.1.into()),
        },
    );
    if res.is_err() {
        eprintln!("Could not copy to clipboard! Make sure you have wl-clipboard installed.");
    }
}

fn paste_latest(items: &mut IndexMap<Vec<u8>, String>) -> (Vec<u8>, String) {
    if items.is_empty() {
        return (Vec::new(), String::from("Empty"));
    }
    let last = items.last().unwrap();
    (last.0.clone(), last.1.clone())
}

fn get_items(items: &mut IndexMap<Vec<u8>, String>) {
    let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Any);
    match result {
        Ok((mut pipe, mimetype)) => {
            let mut contents = vec![];
            pipe.read_to_end(&mut contents)
                .expect("Could not read from pipe");
            if items.get(&contents).is_some() {
                return;
            }
            items.shift_insert(0, contents, mimetype);
        }

        Err(Error::NoSeats) | Err(Error::ClipboardEmpty) | Err(Error::NoMimeType) => {
            // not an error, just a non pipe state
        }

        Err(err) => eprintln!("{}", err),
    }
}

fn start_wl_copy_runner() {
    std::process::Command::new("wl-paste")
        .args([
            // "-p",
            "-w",
            "oxipaste_command_runner",
        ])
        .output()
        .expect("Could not run command runner for wl-copy.");
}
