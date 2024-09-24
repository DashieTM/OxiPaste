use std::os::unix::process::CommandExt;
use std::process::Command;

use config::{create_config, default_config, parse_config, Config};
use context::{
    Address, ContentType, ContentTypeId, ContextMenu, ContextMenuMessage, ImageContext, TextContext,
};
use iced::keyboard::key::Named;
use iced::widget::container::Style;
use iced::widget::{column, container, row, scrollable, Column, Row};
use iced::{event, futures, Alignment, Color, Element, Length, Task, Theme};
use indexmap::IndexMap;
use oxiced::theme::get_theme;
use oxiced::widgets::common::darken_color;
use oxiced::widgets::oxi_button::{button, ButtonVariant};
use oxiced::widgets::oxi_picklist::pick_list;
use oxiced::widgets::oxi_text_input::text_input;

use iced_layershell::actions::LayershellCustomActions;
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::Application;
use zbus::{proxy, Connection};

mod config;
mod context;
mod custom_rich;

pub fn into_general_error(
    error_opt: Option<impl std::error::Error + 'static>,
) -> Option<Box<dyn std::error::Error>> {
    let error = error_opt?;
    Some(Box::new(error) as Box<dyn std::error::Error>)
}

//pub fn main() -> iced::Result {
pub fn main() -> Result<(), iced_layershell::Error> {
    let settings = Settings {
        layer_settings: LayerShellSettings {
            size: Some((600, 600)),
            exclusive_zone: 0,
            anchor: Anchor::Left | Anchor::Right,
            binded_output_name: Some("OxiPaste".into()),
            layer: Layer::Overlay,
            margin: (100, 100, 100, 100),
            ..Default::default()
        },
        ..Default::default()
    };
    Counter::run(settings)
}

struct Counter {
    theme: Theme,
    filter_text: String,
    filter_content_type: ContentTypeId,
    clipboard_content: IndexMap<i32, ContextMenu>,
    proxy: OxiPasteDbusProxy<'static>,
    error_opt: Option<Box<dyn std::error::Error>>,
    config: Config,
}

impl Default for Counter {
    fn default() -> Self {
        // when we don't have a proxy, we have other issues, aka goodbye
        let proxy = futures::executor::block_on(get_connection()).unwrap();
        let data = futures::executor::block_on(get_items(&proxy));
        // TODO use this error
        let (map, _error_opt) = if let Ok(map) = data {
            (map, None)
        } else {
            (IndexMap::new(), into_general_error(data.err()))
        };
        let config_dir = create_config();
        // TODO enable more than one error...
        let (config, error_opt) = if let Ok(dir) = config_dir {
            let config_res = parse_config(&dir);
            if let Ok(config) = config_res {
                (config, None)
            } else {
                (default_config(), config_res.unwrap_err())
            }
        } else {
            (default_config(), config_dir.unwrap_err())
        };
        Self {
            theme: get_theme(),
            filter_text: "".into(),
            filter_content_type: ContentTypeId::All,
            clipboard_content: map,
            proxy, // TODO handle err
            error_opt,
            config, //error_opt: into_general_error(Some(zbus::Error::NameTaken)),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Copy(i32),
    Remove(i32),
    ClearClipboard,
    SetFilterText(String),
    SetContentTypeFilter(ContentTypeId),
    RunContextCommand(Vec<String>, bool, i32),
    SubMessageContext(i32, ContextMenuMessage),
    Exit,
}

impl TryInto<LayershellCustomActions> for Message {
    type Error = Self;
    fn try_into(self) -> Result<LayershellCustomActions, Self::Error> {
        Err(self)
    }
}

fn box_style(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(iced::Background::Color(darken_color(
            palette.background.base.color,
        ))),
        border: iced::border::rounded(10),
        ..container::rounded_box(theme)
    }
}

fn wrap_in_rounded_box<'a>(
    content: impl Into<Element<'a, Message, Theme>>,
) -> Element<'a, Message> {
    container(content)
        .style(box_style)
        .align_x(Alignment::Center)
        .padding(50)
        .max_width(550)
        .into()
}

impl Application for Counter {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Message>) {
        (
            Self {
                ..Default::default()
            },
            Task::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("OxiPaste")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen_with(|event, _status, _id| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                modifiers: _,
                key: iced::keyboard::key::Key::Named(Named::Escape),
                modified_key: _,
                physical_key: _,
                location: _,
                text: _,
            }) => Some(Message::Exit),
            _ => None,
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Copy(value) => {
                let res = futures::executor::block_on(copy_to_clipboard(&self.proxy, value as u32));
                self.error_opt = into_general_error(res.err());
                // TODO make this work with iced exit?
                exit(&self.config);
                Task::none()
            }
            Message::SetFilterText(value) => {
                self.filter_text = value;
                Task::none()
            }
            Message::SetContentTypeFilter(value) => {
                self.filter_content_type = value;
                Task::none()
            }
            Message::Remove(index) => {
                self.clipboard_content.shift_remove(&index);
                Task::none()
            }
            Message::ClearClipboard => {
                let res = futures::executor::block_on(delete_all(&self.proxy));
                self.error_opt = into_general_error(res.err());
                // TODO make this work with iced exit?
                exit(&self.config);
                Task::none()
            }
            Message::RunContextCommand(mut commands, copy, index) => {
                if copy {
                    let res =
                        futures::executor::block_on(copy_to_clipboard(&self.proxy, index as u32));
                    self.error_opt = into_general_error(res.err());
                }
                //TODO this is not safe
                let command = commands.remove(0);
                // TODO handle error?
                let _ = Command::new(command).args(commands).exec();
                exit(&self.config);
                Task::none()
            }
            Message::SubMessageContext(index, ContextMenuMessage::Expand) => {
                let context = self.clipboard_content.get_mut(&index).unwrap();
                context.toggled = !context.toggled;
                Task::none()
            }
            Message::Exit => {
                // TODO make this work with iced exit?
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        wrap_in_rounded_box(window(self))
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    // remove the annoying background color
    fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
        let palette = theme.extended_palette();
        iced_layershell::Appearance {
            background_color: iced::Color::TRANSPARENT,
            text_color: palette.background.base.text,
        }
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

fn clipboard_element<'a>(
    index: i32,
    context: &'a ContextMenu,
    config: &'a Config,
) -> Row<'a, Message> {
    let (content_button, context_button) = context.content_type.get_view_buttons(index);
    if context.toggled {
        // TODO rework this copy
        let (choices, copy) = context.content_type.get_context_actions(config);
        row![
            Row::with_children(choices.into_iter().map(|choice| {
                // TODO not safe
                let mut truncate_string = choice.first().unwrap().clone();
                truncate_string.truncate(5);

                button(iced::widget::text(truncate_string), ButtonVariant::Primary)
                    .on_press(Message::RunContextCommand(choice, copy, index))
                    .into()
            }))
            .spacing(20)
            .width(iced::Length::Fill),
            button("X", ButtonVariant::Primary).on_press(Message::Remove(index)),
            context_button.unwrap()
        ]
        .padding(20)
        .align_y(Alignment::Center)
        .spacing(20)
    } else {
        row![
            content_button
                .width(iced::Length::Fill)
                .on_press(Message::Copy(index)),
            button("X", ButtonVariant::Primary).on_press(Message::Remove(index)),
            if context_button.is_some() {
                row![context_button.unwrap()]
            } else {
                row![]
            },
        ]
        .padding(20)
        .align_y(Alignment::Center)
        .spacing(20)
    }
}

fn window(state: &Counter) -> Column<Message> {
    let elements: Vec<Row<'_, Message>> = state
        .clipboard_content
        .iter()
        .filter(|(_, value)| match &value.content_type {
            ContentType::Text(text_content) => {
                let (text, allow_type) = match text_content {
                    TextContext::Text(text) => (
                        text,
                        (state.filter_content_type == ContentTypeId::All
                            || state.filter_content_type == ContentTypeId::PlainText),
                    ),
                    TextContext::Address(address) => (
                        &address.inner,
                        (state.filter_content_type == ContentTypeId::All
                            || state.filter_content_type == ContentTypeId::AddressText),
                    ),
                };
                text.to_lowercase()
                    .contains(&state.filter_text.to_lowercase())
                    && allow_type
            }
            ContentType::Image(_) => {
                (state.filter_text.contains("image") || state.filter_text.is_empty())
                    && (state.filter_content_type == ContentTypeId::All
                        || state.filter_content_type == ContentTypeId::Image)
            }
        })
        .map(|(key, value)| clipboard_element(*key, value, &state.config))
        .collect();

    let mut elements_col = column![];
    for element in elements {
        elements_col = elements_col.push_maybe(Some(element));
    }
    let elements_scrollable = scrollable(elements_col);

    let error_view = if let Some(error) = &state.error_opt {
        let text: String = error.to_string();
        Some(row![
            iced::widget::text(format!("Error: {}", text)).color(Color::from_rgb(1.0, 0.0, 0.0))
        ])
    } else {
        None
    };

    Column::new()
        .push_maybe(error_view)
        .push(
            column![
                row![
                    pick_list(
                        [
                            ContentTypeId::All,
                            ContentTypeId::PlainText,
                            ContentTypeId::AddressText,
                            ContentTypeId::Image
                        ],
                        Some(state.filter_content_type),
                        Message::SetContentTypeFilter
                    )
                    .width(Length::Fill),
                    button("Clear all", ButtonVariant::Primary).on_press(Message::ClearClipboard)
                ]
                .spacing(10),
                text_input(
                    "Enter text to find",
                    state.filter_text.as_str(),
                    Message::SetFilterText
                ),
            ]
            .padding(20)
            .spacing(20),
        )
        .push(elements_scrollable)
        .padding(10)
        .spacing(20)
        .align_x(Alignment::Center)
}

#[proxy(
    interface = "org.Xetibo.OxiPasteDaemon",
    default_service = "org.Xetibo.OxiPasteDaemon",
    default_path = "/org/Xetibo/OxiPasteDaemon"
)]
#[allow(non_snake_case)]
trait OxiPasteDbus {
    async fn GetAll(&self) -> zbus::Result<Vec<(Vec<u8>, String)>>;
    async fn Paste(&self, index: u32) -> zbus::Result<()>;
    async fn DeleteAll(&self) -> zbus::Result<()>;
}

async fn get_items(proxy: &OxiPasteDbusProxy<'static>) -> zbus::Result<IndexMap<i32, ContextMenu>> {
    let reply = proxy.GetAll().await?;

    let mut map = IndexMap::new();
    for item in reply {
        if item.1.contains("text/plain") {
            let item_res = String::from_utf8(item.0);
            if item_res.is_err() {
                return Err(zbus::Error::Failure(
                    "Could not convert data from daemon".into(),
                ));
            }
            let address_opt = Address::try_build(item_res.unwrap());
            map.insert(
                map.len() as i32,
                ContextMenu {
                    toggled: false,
                    content_type: if let Ok(address) = address_opt {
                        ContentType::Text(TextContext::Address(address))
                    } else {
                        // guaranteed error -> aka text, lmao
                        ContentType::Text(TextContext::Text(address_opt.unwrap_err()))
                    },
                },
            );
        } else {
            map.insert(
                map.len() as i32,
                ContextMenu {
                    toggled: false,
                    content_type: ContentType::Image(ImageContext::Regular(item.0)),
                },
            );
        }
    }
    Ok(map)
}

async fn copy_to_clipboard(proxy: &OxiPasteDbusProxy<'static>, index: u32) -> zbus::Result<()> {
    proxy.Paste(index).await?;
    Ok(())
}

async fn delete_all(proxy: &OxiPasteDbusProxy<'static>) -> zbus::Result<()> {
    proxy.DeleteAll().await?;
    Ok(())
}

async fn get_connection() -> zbus::Result<OxiPasteDbusProxy<'static>> {
    let connection = Connection::session().await?;
    let proxy = OxiPasteDbusProxy::new(&connection).await?;
    Ok(proxy)
}

fn exit(config: &Config) {
    if !config.keepOpen {
        std::process::exit(0);
    }
}
