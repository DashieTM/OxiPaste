use std::os::unix::process::CommandExt;
use std::process::Command;

use context::{context_menu, ContextMenu, ContextMenuMessage, TextContext};
use iced::keyboard::key::Named;
use iced::widget::container::Style;
use iced::widget::{column, container, rich_text, row, scrollable, span, Column, Row};
use iced::{event, futures, Alignment, Element, Task, Theme};
use indexmap::IndexMap;
use oxiced::theme::get_theme;
use oxiced::widgets::common::darken_color;
use oxiced::widgets::oxi_button::{button, ButtonVariant};
use oxiced::widgets::oxi_text_input::text_input;

use iced_layershell::actions::LayershellCustomActions;
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::Application;
use zbus::{proxy, Connection};

mod context;
mod custom_rich;
use custom_rich::CustomRich;

//pub fn main() -> iced::Result {
pub fn main() -> Result<(), iced_layershell::Error> {
    let settings = Settings {
        layer_settings: LayerShellSettings {
            size: Some((600, 600)),
            exclusive_zone: 0,
            anchor: Anchor::Left | Anchor::Right,
            binded_output_name: Some("pingpang".into()),
            layer: Layer::Overlay,
            margin: (100, 100, 100, 100),
            ..Default::default()
        },
        ..Default::default()
    };
    Counter::run(settings)
}

#[derive(Debug, Clone)]
enum ContentType {
    Text(TextContext),
    Image(Vec<u8>),
}

struct Counter {
    theme: Theme,
    filter_text: String,
    clipboard_content: IndexMap<i32, (ContentType, ContextMenu)>,
    proxy: OxiPasteDbusProxy<'static>,
}

impl Default for Counter {
    fn default() -> Self {
        let proxy = futures::executor::block_on(get_connection()).unwrap();
        let data = futures::executor::block_on(get_items(&proxy));
        let map = if let Ok(map) = data {
            map
        } else {
            IndexMap::new()
        };
        Self {
            theme: get_theme(),
            filter_text: "".into(),
            clipboard_content: map,
            proxy, // TODO handle err
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Copy(i32),
    Remove(i32),
    ClearClipboard,
    SetFilterText(String),
    RunContextCommand(Vec<String>),
    ContextMenuMessage(i32, ContextMenuMessage),
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
        String::from("Oxiced")
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
                let _ = futures::executor::block_on(copy_to_clipboard(&self.proxy, value as u32));
                // TODO make this work with iced exit?
                std::process::exit(0);
            }
            Message::SetFilterText(value) => {
                self.filter_text = value;
                Task::none()
            }
            Message::Remove(index) => {
                self.clipboard_content.shift_remove(&index);
                Task::none()
            }
            Message::ClearClipboard => {
                let _ = futures::executor::block_on(delete_all(&self.proxy));
                // TODO make this work with iced exit?
                std::process::exit(0);
            }
            Message::RunContextCommand(mut commands) => {
                //TODO this is not safe
                let first = commands.remove(0);
                let res = Command::new(first).args(commands).exec();
                dbg!(res);
                std::process::exit(0);
            }
            Message::ContextMenuMessage(index, ContextMenuMessage::Expand) => {
                let context = self.clipboard_content.get_mut(&index).unwrap();
                context.1.toggled = !context.1.toggled;
                Task::none()
            }
            Message::Exit => {
                // TODO make this work with iced exit?
                std::process::exit(0);
            }
            _ => Task::none(),
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
    contents: &ContentType,
    context: &ContextMenu,
) -> Row<'a, Message> {
    let (content_button, context_button) = match contents {
        ContentType::Text(text_content) => match text_content {
            TextContext::Text(text) => (
                button(
                    CustomRich::custom_rich(rich_text![span(text.to_owned()).underline(false)]),
                    ButtonVariant::Secondary,
                ),
                Some(context_menu(
                    context,
                    text_content.get_context_actions(),
                    index,
                )),
            ),
            TextContext::Address(_) => todo!(),
        },
        ContentType::Image(image_content) => {
            let handle = iced::widget::image::Handle::from_bytes(image_content.clone());
            (
                button(iced::widget::image(handle), ButtonVariant::Secondary),
                None,
            )
        }
    };
    row![
        content_button
            .padding([20, 5])
            .width(iced::Length::Fill)
            .on_press(Message::Copy(index)),
        button("X", ButtonVariant::Primary)
            .on_press(Message::Remove(index))
            //.align_y(Alignment::Center)
            .padding([20, 5]),
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

fn window<'a>(state: &Counter) -> Column<'a, Message> {
    let elements: Vec<Row<'_, Message>> = state
        .clipboard_content
        .iter()
        .filter(|(_, value)| match &value.0 {
            ContentType::Text(text_content) => match text_content {
                TextContext::Text(text) => text
                    .to_lowercase()
                    .contains(&state.filter_text.to_lowercase()),
                TextContext::Address(_) => todo!(),
            },
            ContentType::Image(_) => {
                state.filter_text.contains("image") || state.filter_text.is_empty()
            }
        })
        .map(|(key, value)| clipboard_element(*key, &value.0, &value.1))
        .collect();

    let mut elements_col = column![];
    for element in elements {
        elements_col = elements_col.push_maybe(Some(element));
    }
    let elements_scrollable = scrollable(elements_col);

    column![
        row![
            text_input(
                "Enter text to find",
                state.filter_text.as_str(),
                Message::SetFilterText
            ),
            button("Clear all", ButtonVariant::Primary).on_press(Message::ClearClipboard)
        ]
        .padding(20)
        .spacing(20),
        elements_scrollable
    ]
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

async fn get_items(
    proxy: &OxiPasteDbusProxy<'static>,
) -> zbus::Result<IndexMap<i32, (ContentType, ContextMenu)>> {
    let reply = proxy.GetAll().await?;

    let mut map = IndexMap::new();
    for item in reply {
        if item.1.contains("text/plain") {
            map.insert(
                map.len() as i32,
                (
                    ContentType::Text(TextContext::Text(String::from_utf8(item.0).unwrap())),
                    ContextMenu { toggled: false },
                ),
            );
        } else {
            map.insert(
                map.len() as i32,
                (ContentType::Image(item.0), ContextMenu { toggled: false }),
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
