use std::fmt::Display;

use iced::widget::{rich_text, span, Button};
use oxiced::widgets::oxi_button::{button, ButtonVariant};

use crate::{custom_rich::CustomRich, Message};

#[derive(Debug, Clone)]
pub struct Address {
    inner: String,
}

#[allow(dead_code)]
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

#[derive(Debug, Clone)]
pub enum ImageContext {
    Regular(Vec<u8>),
}

impl ImageContext {
    pub fn get_view_buttons(&self, _: i32) -> (Button<Message>, Option<Button<Message>>) {
        match self {
            Self::Regular(image_content) => {
                let handle = iced::widget::image::Handle::from_bytes(image_content.clone());
                (
                    button(iced::widget::image(handle), ButtonVariant::Secondary),
                    None,
                )
            }
        }
    }
    pub fn get_context_actions(&self) -> Vec<Vec<String>> {
        match self {
            Self::Regular(_) => Vec::new(),
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
    pub fn get_view_buttons(&self, index: i32) -> (Button<Message>, Option<Button<Message>>) {
        match self {
            TextContext::Address(_) => todo!(),
            TextContext::Text(text) => {
                (
                    button(
                        CustomRich::custom_rich(rich_text![span(text.to_owned()).underline(false)]),
                        ButtonVariant::Secondary,
                    ),
                    Some(button("...", ButtonVariant::Primary).on_press(
                        Message::SubMessageContext(index, ContextMenuMessage::Expand),
                    )),
                )
            }
        }
    }
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
    pub fn get_context_actions(&self) -> Vec<Vec<String>> {
        match self {
            ContentType::Text(context) => context.get_context_actions(),
            ContentType::Image(context) => context.get_context_actions(),
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
