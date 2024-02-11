pub mod dbus;
use gtk::glib::unlink;
use std::os::linux::fs;
use std::os::unix::net::UnixListener;
use std::process::Stdio;
use std::sync::mpsc;
use std::{io::Read, path::Path};
use wl_clipboard_rs::copy::{Options, Source};
use wl_clipboard_rs::paste::{get_contents, ClipboardType, Error, MimeType, Seat};

pub enum ReverseCommand {
    SendLatest((Vec<u8>, String)),
    SendAll(Vec<(Vec<u8>, String)>),
}

pub enum Command {
    ShutDown,
    Copy,
    DeleteAtIndex(usize),
    DeleteAll,
    GetLatest,
    GetAll,
    Paste(usize),
    PasteAndDelete(usize),
}

fn main() {
    std::thread::spawn(|| {
        start_wl_copy_runner();
    });
    let (sender, receiver) = mpsc::channel::<Command>();
    let (reverse_sender, reverse_receiver) = mpsc::channel::<ReverseCommand>();
    std::thread::spawn(move || {
        dbus::run(sender, reverse_receiver);
    });
    let mut items: Vec<(Vec<u8>, String)> = Vec::new();
    loop {
        // get_items(&mut items);
        let result = receiver.recv();
        let len = items.len();
        // clean memory in order to not leak
        // can later be configured with config file or something
        if len > 100 {
            let (_, second) = items.split_at_mut(len / 2);
            items = second.to_vec();
        }
        if let Ok(command) = result {
            match command {
                Command::ShutDown => break,
                Command::Copy => get_items(&mut items),
                Command::DeleteAtIndex(index) => {
                    items.remove(index);
                }
                Command::DeleteAll => items.clear(),
                Command::GetLatest => reverse_sender
                    .send(ReverseCommand::SendLatest(paste_latest(&mut items)))
                    .expect("wat"),
                Command::GetAll => reverse_sender
                    .send(ReverseCommand::SendAll(items.clone()))
                    .expect("wat"),
                Command::Paste(index) => copy_to_clipboard(&items, index),
                Command::PasteAndDelete(index) => {
                    copy_to_clipboard(&items, index);
                    items.remove(index);
                }
            }
        }
    }
}

fn copy_to_clipboard(items: &Vec<(Vec<u8>, String)>, index: usize) {
    let item = items.get(index);
    if item.is_none() {
        return;
    }
    let item = item.unwrap();
    let mut opts = Options::new();
    opts.trim_newline(true);
    opts.clipboard(wl_clipboard_rs::copy::ClipboardType::Regular);
    opts.copy(
        Source::Bytes(item.0.clone().into()),
        wl_clipboard_rs::copy::MimeType::Autodetect,
    );
}

fn paste_latest(items: &mut Vec<(Vec<u8>, String)>) -> (Vec<u8>, String) {
    if items.is_empty() {
        return (Vec::new(), String::from("Empty"));
    }
    items.last().unwrap().clone()
}

fn get_items(items: &mut Vec<(Vec<u8>, String)>) {
    let result = get_contents(ClipboardType::Primary, Seat::Unspecified, MimeType::Any);
    match result {
        Ok((mut pipe, mimetype)) => {
            println!("type: {}", &mimetype);
            let mut contents = vec![];
            pipe.read_to_end(&mut contents).expect("grengeng");
            if items.contains(&(contents.clone(), mimetype.clone())) {
                return;
            }
            items.push((contents, mimetype));
            // wl_clipboard_rs::copy::clear(clipboard, seat)
        }

        Err(Error::NoSeats) | Err(Error::ClipboardEmpty) | Err(Error::NoMimeType) => {
            // The clipboard is empty or doesn't contain text, nothing to worry about.
        }

        Err(err) => println!("Error: {}", err),
    }
}

fn start_wl_copy_runner() {
    std::process::Command::new("wl-paste")
        .args([
            "-p",
            "-w",
            "/home/dashie/gits/OxiPaste/target/release/command_runner",
        ])
        .output()
        .expect("Could not run command runner for wl-copy.");
}
