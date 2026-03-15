use std::sync::Arc;

use iced::{
    Color, Element, Length, Size, Subscription, Task,
    event,
    keyboard::{Event as KeyEvent, Key, key::Named},
    widget::{column, container, image, row, scrollable, svg, text, text_input, Space},
    window,
};

use k_launcher_kernel::{Kernel, SearchResult};

use crate::theme;

static INPUT_ID: std::sync::LazyLock<iced::widget::Id> =
    std::sync::LazyLock::new(|| iced::widget::Id::new("search"));

pub struct KLauncherApp {
    kernel: Arc<Kernel>,
    query: String,
    results: Arc<Vec<SearchResult>>,
    selected: usize,
}

impl KLauncherApp {
    fn new(kernel: Arc<Kernel>) -> Self {
        Self {
            kernel,
            query: String::new(),
            results: Arc::new(vec![]),
            selected: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    ResultsReady(Arc<Vec<SearchResult>>),
    KeyPressed(KeyEvent),
}

fn update(state: &mut KLauncherApp, message: Message) -> Task<Message> {
    match message {
        Message::QueryChanged(q) => {
            state.query = q.clone();
            state.selected = 0;
            let kernel = state.kernel.clone();
            Task::perform(
                async move { kernel.search(&q).await },
                |results| Message::ResultsReady(Arc::new(results)),
            )
        }
        Message::ResultsReady(results) => {
            state.results = results;
            Task::none()
        }
        Message::KeyPressed(event) => {
            let key = match event {
                KeyEvent::KeyPressed { key, .. } => key,
                _ => return Task::none(),
            };
            let Key::Named(named) = key else {
                return Task::none();
            };
            let len = state.results.len();
            match named {
                Named::Escape => std::process::exit(0),
                Named::ArrowDown => {
                    if len > 0 {
                        state.selected = (state.selected + 1).min(len - 1);
                    }
                }
                Named::ArrowUp => {
                    if state.selected > 0 {
                        state.selected -= 1;
                    }
                }
                Named::Enter => {
                    if let Some(result) = state.results.get(state.selected) {
                        (result.on_execute)();
                    }
                    std::process::exit(0);
                }
                _ => {}
            }
            Task::none()
        }
    }
}

fn view(state: &KLauncherApp) -> Element<'_, Message> {
    let colors = &*theme::AERO;

    let search_bar = text_input("Search...", &state.query)
        .id(INPUT_ID.clone())
        .on_input(Message::QueryChanged)
        .padding(12)
        .size(18);

    let result_rows: Vec<Element<'_, Message>> = state
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_selected = i == state.selected;
            let bg_color = if is_selected {
                colors.border_cyan
            } else {
                Color::from_rgba8(255, 255, 255, 0.07)
            };
            let icon_el: Element<'_, Message> = match &result.icon {
                Some(p) if p.ends_with(".svg") =>
                    svg(svg::Handle::from_path(p)).width(24).height(24).into(),
                Some(p) =>
                    image(image::Handle::from_path(p)).width(24).height(24).into(),
                None => Space::new().width(24).height(24).into(),
            };
            let title_col: Element<'_, Message> = if let Some(desc) = &result.description {
                column![
                    text(result.title.as_str()).size(15),
                    text(desc).size(11).color(Color::from_rgba8(180, 180, 200, 0.8)),
                ]
                .into()
            } else {
                text(result.title.as_str()).size(15).into()
            };
            container(
                row![icon_el, title_col]
                    .spacing(8)
                    .align_y(iced::Center),
            )
                .width(Length::Fill)
                .padding([6, 12])
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(bg_color)),
                    ..Default::default()
                })
                .into()
        })
        .collect();

    let results_list =
        scrollable(column(result_rows).spacing(2).width(Length::Fill)).height(Length::Fill);

    let content = column![search_bar, results_list]
        .spacing(8)
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba8(
                20, 20, 30, 0.9,
            ))),
            ..Default::default()
        })
        .into()
}

fn subscription(_state: &KLauncherApp) -> Subscription<Message> {
    event::listen_with(|ev, _status, _id| match ev {
        iced::Event::Keyboard(ke) => Some(Message::KeyPressed(ke)),
        _ => None,
    })
}

pub fn run(kernel: Arc<Kernel>) -> iced::Result {
    iced::application(
        move || {
            let app = KLauncherApp::new(kernel.clone());
            let focus = iced::widget::operation::focus(INPUT_ID.clone());
            (app, focus)
        },
        update,
        view,
    )
        .title("K-Launcher")
        .subscription(subscription)
        .window(window::Settings {
            size: Size::new(600.0, 400.0),
            position: window::Position::Centered,
            decorations: false,
            transparent: true,
            resizable: false,
            ..Default::default()
        })
        .run()
}
