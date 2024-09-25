use std::{fmt::Display, process::Command};

use iced::{
    futures,
    widget::{rich_text, span, Button},
};
use oxiced::widgets::oxi_button::{button, ButtonVariant};

use crate::{
    config::Config, copy_to_clipboard, custom_rich::CustomRich, into_general_error, Message,
    OxiPaste,
};

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
    //
    fn apply_value(
        commands: Vec<Vec<String>>,
    ) -> Vec<Result<ContextCommand, Box<dyn std::error::Error>>> {
        commands
            .into_iter()
            .map(|command| {
                let res = ContextCommand::from_vec_and_value(command, "", false)?;
                Ok(res)
            })
            .collect()
    }

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
    pub fn get_context_actions(
        &self,
        config: &Config,
    ) -> Vec<Result<ContextCommand, Box<dyn std::error::Error>>> {
        match self {
            // TODO perhaps copy to clipboard here instead?
            // right now it requires a call to dbus...
            Self::Regular(_) => ImageContext::apply_value(config.ImageContextActions.clone()),
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
    fn apply_value(
        commands: Vec<Vec<String>>,
        value: &str,
    ) -> Vec<Result<ContextCommand, Box<dyn std::error::Error>>> {
        commands
            .into_iter()
            .map(|command| {
                let res = ContextCommand::from_vec_and_value(command, value, true)?;
                Ok(res)
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
    pub fn get_context_actions(
        &self,
        config: &Config,
    ) -> Vec<Result<ContextCommand, Box<dyn std::error::Error>>> {
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

pub struct GetContextActionsResult(
    pub Vec<Result<ContextCommand, Box<dyn std::error::Error>>>,
    pub bool,
);

impl ContentType {
    pub fn get_view_buttons(&self, index: i32) -> (Button<Message>, Option<Button<Message>>) {
        match self {
            ContentType::Text(context) => context.get_view_buttons(index),
            ContentType::Image(context) => context.get_view_buttons(index),
        }
    }
    pub fn get_context_actions(&self, config: &Config) -> GetContextActionsResult {
        match self {
            ContentType::Text(context) => {
                GetContextActionsResult(context.get_context_actions(config), false)
            }
            ContentType::Image(context) => {
                GetContextActionsResult(context.get_context_actions(config), true)
            }
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

#[derive(Debug, Clone)]
pub struct ContextCommand {
    pub label: String,
    pub command: String,
    pub args: Vec<String>,
    pub requires_copy: bool,
}

impl ContextCommand {
    pub fn from_vec_and_value(
        mut args: Vec<String>,
        value: &str,
        requires_copy: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if args.len() < 2 {
            return Err(Box::new(zbus::Error::Failure(
                "Invalid Command: less than 2 arguments provided".into(),
            )));
        }
        let label = args.remove(0);
        let command = args.remove(0);
        let mut found = false;
        for arg in args.iter_mut() {
            if arg == "{}" {
                *arg = value.to_owned();
                found = true;
            }
        }
        if !found {
            args.push(value.to_owned());
        }
        Ok(Self {
            label,
            command,
            args,
            requires_copy,
        })
    }

    pub fn run_command(
        &self,
        oxipaste: &OxiPaste,
        index: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.requires_copy {
            let res = futures::executor::block_on(copy_to_clipboard(&oxipaste.proxy, index as u32));
            let err_opt = into_general_error(res.err());
            if let Some(error) = err_opt {
                return Err(error);
            }
        }
        let res = Command::new(&self.command).args(&self.args).spawn();
        if let Some(error) = into_general_error(res.err()) {
            Err(error)
        } else {
            Ok(())
        }
    }
}
