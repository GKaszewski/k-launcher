use std::sync::Arc;

use iced::{
    Border, Color, Element, Length, Size, Subscription, Task,
    event,
    keyboard::{Event as KeyEvent, Key, key::Named},
    widget::{column, container, image, row, scrollable, svg, text, text_input, Space},
    window,
};

use k_launcher_kernel::{AppLauncher, SearchEngine, SearchResult};
use k_launcher_os_bridge::WindowConfig;

use crate::theme;

static INPUT_ID: std::sync::LazyLock<iced::widget::Id> =
    std::sync::LazyLock::new(|| iced::widget::Id::new("search"));

pub struct KLauncherApp {
    engine: Arc<dyn SearchEngine>,
    launcher: Arc<dyn AppLauncher>,
    query: String,
    results: Arc<Vec<SearchResult>>,
    selected: usize,
}

impl KLauncherApp {
    fn new(engine: Arc<dyn SearchEngine>, launcher: Arc<dyn AppLauncher>) -> Self {
        Self {
            engine,
            launcher,
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
            let engine = state.engine.clone();
            Task::perform(
                async move { engine.search(&q).await },
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
                        if let Some(on_select) = &result.on_select {
                            on_select();
                        }
                        state.launcher.execute(&result.action);
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
                    text(desc).size(12).color(Color::from_rgba8(210, 215, 230, 1.0)),
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
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        })
        .collect();

    let results_list = if state.results.is_empty() && !state.query.is_empty() {
        scrollable(
            container(
                text("No results")
                    .size(15)
                    .color(Color::from_rgba8(180, 180, 200, 0.5)),
            )
            .width(Length::Fill)
            .align_x(iced::Center)
            .padding([20, 0]),
        )
        .height(Length::Fill)
    } else {
        scrollable(column(result_rows).spacing(2).width(Length::Fill)).height(Length::Fill)
    };

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
            border: Border {
                color: theme::AERO.border_cyan,
                width: 1.0,
                radius: 8.0.into(),
            },
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

pub fn run(engine: Arc<dyn SearchEngine>, launcher: Arc<dyn AppLauncher>) -> iced::Result {
    let wc = WindowConfig::launcher();
    iced::application(
        move || {
            let app = KLauncherApp::new(engine.clone(), launcher.clone());
            let focus = iced::widget::operation::focus(INPUT_ID.clone());
            (app, focus)
        },
        update,
        view,
    )
        .title("K-Launcher")
        .subscription(subscription)
        .window(window::Settings {
            size: Size::new(wc.width, wc.height),
            position: window::Position::Centered,
            decorations: wc.decorations,
            transparent: wc.transparent,
            resizable: wc.resizable,
            ..Default::default()
        })
        .run()
}
