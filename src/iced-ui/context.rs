use std::fmt::Display;

use iced::widget::{rich_text, span, Button};
use oxiced::widgets::oxi_button::{button, ButtonVariant};

use crate::{config::Config, custom_rich::CustomRich, Message};

#[derive(Debug, Clone)]
pub struct Address {
    pub(crate) inner: String,
}

impl Address {
    pub fn try_build(value: String) -> Result<Self, String> {
        if value.starts_with("https://") || value.starts_with("/") || value.starts_with("./") {
            Ok(Self { inner: value })
        } else {
            // abuse of error lul
            Err(value)
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Clone)]
pub enum ImageContext {
    Regular(Vec<u8>),
}

impl ImageContext {
    // TODO remove when not needed
    //fn apply_value(commands: Vec<Vec<String>>, value: &[u8]) -> Vec<Vec<String>> {
    //    commands
    //        .into_iter()
    //        .map(|mut command| {
    //            // TODO handle this error instead
    //            command.push(String::from_utf8_lossy(value).into());
    //            command
    //        })
    //        .collect()
    //}

    // TODO fix duplication
    pub fn get_view_buttons(&self, index: i32) -> (Button<Message>, Option<Button<Message>>) {
        match self {
            Self::Regular(image_content) => {
                let handle = iced::widget::image::Handle::from_bytes(image_content.clone());
                (
                    button(iced::widget::image(handle), ButtonVariant::Secondary),
                    Some(button("...", ButtonVariant::Primary).on_press(
                        Message::SubMessageContext(index, ContextMenuMessage::Expand),
                    )),
                )
            }
        }
    }
    pub fn get_context_actions(&self, config: &Config) -> Vec<Vec<String>> {
        match self {
            // TODO perhaps copy to clipboard here instead?
            // right now it requires a call to dbus...
            Self::Regular(_) => config.ImageContextActions.clone(),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TextContext {
    Address(Address),
    Text(String),
}

impl TextContext {
    fn apply_value(commands: Vec<Vec<String>>, value: &String) -> Vec<Vec<String>> {
        commands
            .into_iter()
            .map(|mut command| {
                command.push(value.clone());
                command
            })
            .collect()
    }

    pub fn get_view_buttons(&self, index: i32) -> (Button<Message>, Option<Button<Message>>) {
        let text = match self {
            TextContext::Address(address) => &address.inner,
            TextContext::Text(text) => text,
        };
        (
            button(
                CustomRich::custom_rich(rich_text![span(text.to_owned()).underline(false)]),
                ButtonVariant::Secondary,
            ),
            Some(
                button("...", ButtonVariant::Primary).on_press(Message::SubMessageContext(
                    index,
                    ContextMenuMessage::Expand,
                )),
            ),
        )
    }
    // improve this via using actions instead
    pub fn get_context_actions(&self, config: &Config) -> Vec<Vec<String>> {
        match self {
            TextContext::Address(address) => {
                Self::apply_value(config.AddressContextActions.clone(), &address.inner)
            }
            TextContext::Text(text) => {
                Self::apply_value(config.PlainTextContextActions.clone(), text)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ContentTypeId {
    PlainText,
    AddressText,
    Image,
    All,
}

impl Display for ContentTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text: String = match self {
            ContentTypeId::PlainText => "Text".into(),
            ContentTypeId::AddressText => "Addresses".into(),
            ContentTypeId::Image => "Images".into(),
            ContentTypeId::All => "All".into(),
        };
        write!(f, "{}", text)
    }
}

#[derive(Debug, Clone)]
pub enum ContentType {
    Text(TextContext),
    Image(ImageContext),
}

impl ContentType {
    pub fn get_view_buttons(&self, index: i32) -> (Button<Message>, Option<Button<Message>>) {
        match self {
            ContentType::Text(context) => context.get_view_buttons(index),
            ContentType::Image(context) => context.get_view_buttons(index),
        }
    }
    pub fn get_context_actions(&self, config: &Config) -> (Vec<Vec<String>>, bool) {
        match self {
            ContentType::Text(context) => (context.get_context_actions(config), false),
            ContentType::Image(context) => (context.get_context_actions(config), true),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub(crate) toggled: bool,
    pub(crate) content_type: ContentType,
}

#[derive(Debug, Clone)]
pub enum ContextMenuMessage {
    Expand,
}
