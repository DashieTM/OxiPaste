use std::sync::mpsc::{Receiver, Sender};

use crate::{Command, ReverseCommand};

use std::{error::Error, future::pending};
use zbus::{connection, interface};

struct OxiPasteDbus {
    sender: Sender<Command>,
    receiver: Receiver<ReverseCommand>,
}

unsafe impl Send for OxiPasteDbus {}
unsafe impl Sync for OxiPasteDbus {}

#[interface(name = "org.Xetibo.OxiPasteDaemon")]
#[allow(non_snake_case)]
impl OxiPasteDbus {
    fn Copy(&mut self) {
        let _ = self.sender.send(Command::Copy);
    }
    fn Paste(&mut self, index: u32) {
        let _ = self.sender.send(Command::Paste(index as usize));
    }
    fn PasteAndDelete(&mut self, index: u32) {
        let _ = self.sender.send(Command::PasteAndDelete(index as usize));
    }
    fn GetAll(&mut self) -> Vec<(Vec<u8>, String)> {
        let mut response = Vec::new();
        self.sender
            .send(Command::GetAll)
            .expect("Could not send command");
        let res = self.receiver.recv();
        if let Ok(ReverseCommand::SendAll(items)) = res {
            response = items;
        }
        response
    }
    fn GetLatest(&mut self) -> (Vec<u8>, String) {
        let (mut response, mut mimetype) = (Vec::new(), String::from("Empty"));
        self.sender
            .send(Command::GetLatest)
            .expect("Could not send command");
        let res = self.receiver.recv();
        if let Ok(ReverseCommand::SendLatest(text)) = res {
            response = text.0;
            mimetype = text.1;
        }
        (response, mimetype)
    }
    fn DeleteAtIndex(&mut self, index: u32) {
        let _ = self.sender.send(Command::DeleteAtIndex(index as usize));
    }
    fn DeleteAll(&mut self) {
        let _ = self.sender.send(Command::DeleteAll);
    }
    fn ShutDown(&mut self) {
        let _ = self.sender.send(Command::ShutDown);
    }
}

pub async fn run(
    sender: Sender<Command>,
    receiver: Receiver<ReverseCommand>,
) -> Result<(), Box<dyn Error>> {
    let oxipaste_dbus = OxiPasteDbus { sender, receiver };
    let _conn = connection::Builder::session()?
        .name("org.Xetibo.OxiPasteDaemon")?
        .serve_at("/org/Xetibo/OxiPasteDaemon", oxipaste_dbus)?
        .build()
        .await?;

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
