use std::fmt::Display;

use iced::{
    overlay::menu::Catalog,
    widget::{Button, Column, Row, Text},
    Element, Task, Theme,
};
use iced_aw::drop_down;

use crate::Message;

#[derive(Debug, Clone)]
pub struct Address {
    inner: String,
}

impl Address {
    pub fn try_build(value: String) -> Option<Self> {
        if value.starts_with("https::/") {
            Some(Self { inner: value })
        } else {
            None
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

enum PictureContext {}

#[derive(Debug, Clone)]
pub enum TextContext {
    Address(Address),
    Text(String),
}

impl TextContext {
    pub fn get_context_actions(&self) -> Vec<Vec<String>> {
        match self {
            TextContext::Address(address) => {
                vec![
                    vec!["xdg-open".into(), address.inner.clone()],
                    vec!["echo".into(), address.inner.clone()],
                ]
            }
            TextContext::Text(text) => {
                vec![
                    vec!["notify-send".into(), text.clone()],
                    vec!["echo".into(), text.clone()],
                ]
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub(crate) toggled: bool,
}

#[derive(Debug, Clone)]
pub enum ContextMenuMessage {
    Expand,
    Select(String),
}

pub fn context_menu<'a>(
    menu: &ContextMenu,
    choices: Vec<Vec<String>>,
    index: i32,
) -> Element<'a, Message> {
    let underlay = Row::new().push(Button::new(Text::new("expand")).on_press(
        Message::ContextMenuMessage(index, ContextMenuMessage::Expand),
    ));

    let overlay = Column::with_children(choices.into_iter().map(|choice| {
        // TODO not safe
        let mut truncate_string = choice.first().unwrap().clone();
        truncate_string.truncate(5);
        Row::new()
            .push(Text::new(truncate_string))
            .push(Button::new(Text::new("choose")).on_press(Message::RunContextCommand(choice)))
            .into()
    }));
    println!("{}", menu.toggled);
    drop_down::DropDown::new(underlay, overlay, menu.toggled).into()
}
