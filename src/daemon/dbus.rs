use dbus::blocking::Connection;
use std::sync::mpsc::{Receiver, Sender};

use crate::{Command, ReverseCommand};

pub fn run(sender: Sender<Command>, receiver: Receiver<ReverseCommand>) {
    let c = Connection::new_session().unwrap();
    c.request_name("org.Xetibo.OxiPasteDaemon", false, true, false)
        .unwrap();
    let mut cr = dbus_crossroads::Crossroads::new();
    let token = cr.register("org.Xetibo.OxiPasteDaemon", |c| {
        c.method(
            "Copy",
            (),
            ("reply",),
            move |_, data: &mut DaemonData, ()| {
                let res = data.sender.send(Command::Copy).is_ok();
                Ok((res,))
            },
        );
        c.method(
            "Paste",
            ("index",),
            ("reply",),
            move |_, data: &mut DaemonData, (index,): (u32,)| {
                let res = data.sender.send(Command::Paste(index as usize)).is_ok();
                Ok((res,))
            },
        );
        c.method(
            "PasteAndDelete",
            ("index",),
            ("reply",),
            move |_, data: &mut DaemonData, (index,): (u32,)| {
                let res = data.sender.send(Command::PasteAndDelete(index as usize)).is_ok();
                Ok((res,))
            },
        );
        c.method(
            "GetAll",
            (),
            ("reply",),
            move |_, data: &mut DaemonData, ()| {
                let mut response = Vec::new();
                data.sender.send(Command::GetAll).expect("peng");
                let res = data.receiver.recv();
                if let Ok(ReverseCommand::SendAll(items)) = res {
                    response = items;
                }
                Ok((response,))
            },
        );
        c.method(
            "GetLatest",
            (),
            ("reply",),
            move |_, data: &mut DaemonData, ()| {
                let (mut response, mut mimetype) = (Vec::new(), String::from("Empty"));
                data.sender.send(Command::GetLatest).expect("peng");
                let res = data.receiver.recv();
                if let Ok(ReverseCommand::SendLatest(text)) = res {
                    response = text.0;
                    mimetype = text.1;
                }
                Ok(((response, mimetype),))
            },
        );
        c.method(
            "DeleteAtIndex",
            ("index",),
            ("reply",),
            move |_, data: &mut DaemonData, (index,): (u32,)| {
                let response = data
                    .sender
                    .send(Command::DeleteAtIndex(index as usize))
                    .is_ok();
                Ok((response,))
            },
        );
        c.method(
            "DeleteAll",
            (),
            ("reply",),
            move |_, data: &mut DaemonData, ()| {
                let response = data.sender.send(Command::DeleteAll).is_ok();
                Ok((response,))
            },
        );
        c.method(
            "ShutDown",
            (),
            ("reply",),
            move |_, data: &mut DaemonData, ()| {
                let res = data.sender.send(Command::ShutDown).is_ok();
                Ok((res,))
            },
        );
    });
    cr.insert(
        "/org/Xetibo/OxiPasteDaemon",
        &[token],
        DaemonData { sender, receiver },
    );
    cr.serve(&c).unwrap();
}

pub struct DaemonData {
    sender: Sender<Command>,
    receiver: Receiver<ReverseCommand>,
}
