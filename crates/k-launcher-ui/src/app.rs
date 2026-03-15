use std::sync::Arc;

use iced::{
    Border, Color, Element, Length, Size, Subscription, Task, event,
    keyboard::{Event as KeyEvent, Key, key::Named},
    widget::{Space, column, container, image, row, scrollable, svg, text, text_input},
    window,
};

use k_launcher_config::AppearanceCfg;
use k_launcher_kernel::{AppLauncher, SearchEngine, SearchResult};
use k_launcher_os_bridge::WindowConfig;

static INPUT_ID: std::sync::LazyLock<iced::widget::Id> =
    std::sync::LazyLock::new(|| iced::widget::Id::new("search"));

fn rgba(c: &[f32; 4]) -> Color {
    Color::from_rgba8(c[0] as u8, c[1] as u8, c[2] as u8, c[3])
}

pub struct KLauncherApp {
    engine: Arc<dyn SearchEngine>,
    launcher: Arc<dyn AppLauncher>,
    query: String,
    results: Arc<Vec<SearchResult>>,
    selected: usize,
    cfg: AppearanceCfg,
    error: Option<String>,
    search_epoch: u64,
}

impl KLauncherApp {
    fn new(
        engine: Arc<dyn SearchEngine>,
        launcher: Arc<dyn AppLauncher>,
        cfg: AppearanceCfg,
    ) -> Self {
        Self {
            engine,
            launcher,
            query: String::new(),
            results: Arc::new(vec![]),
            selected: 0,
            cfg,
            error: None,
            search_epoch: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    ResultsReady(u64, Arc<Vec<SearchResult>>),
    KeyPressed(KeyEvent),
}

fn update(state: &mut KLauncherApp, message: Message) -> Task<Message> {
    match message {
        Message::QueryChanged(q) => {
            state.error = None;
            state.query = q.clone();
            state.selected = 0;
            state.search_epoch += 1;
            let epoch = state.search_epoch;
            let engine = state.engine.clone();
            Task::perform(
                async move {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    (epoch, engine.search(&q).await)
                },
                |(epoch, results)| Message::ResultsReady(epoch, Arc::new(results)),
            )
        }
        Message::ResultsReady(epoch, results) => {
            if epoch == state.search_epoch {
                state.results = results;
            }
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
                Named::Escape => {
                    std::process::exit(0);
                }
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
    let cfg = &state.cfg;
    let border_color = rgba(&cfg.border_rgba);

    let search_bar = text_input(&cfg.placeholder, &state.query)
        .id(INPUT_ID.clone())
        .on_input(Message::QueryChanged)
        .padding(12)
        .size(cfg.search_font_size)
        .style(|theme, _status| {
            let mut s =
                iced::widget::text_input::default(theme, iced::widget::text_input::Status::Active);
            s.border = Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            };
            s
        });

    let row_radius: f32 = cfg.row_radius;
    let title_size: f32 = cfg.title_size;
    let desc_size: f32 = cfg.desc_size;

    let result_rows: Vec<Element<'_, Message>> = state
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_selected = i == state.selected;
            let bg_color = if is_selected {
                border_color
            } else {
                Color::from_rgba8(255, 255, 255, 0.07)
            };
            let icon_el: Element<'_, Message> = match &result.icon {
                Some(p) if p.ends_with(".svg") => {
                    svg(svg::Handle::from_path(p)).width(24).height(24).into()
                }
                Some(p) => image(image::Handle::from_path(p))
                    .width(24)
                    .height(24)
                    .into(),
                None => Space::new().width(24).height(24).into(),
            };
            let title_col: Element<'_, Message> = if let Some(desc) = &result.description {
                column![
                    text(result.title.as_str()).size(title_size),
                    text(desc)
                        .size(desc_size)
                        .color(Color::from_rgba8(210, 215, 230, 1.0)),
                ]
                .into()
            } else {
                text(result.title.as_str()).size(title_size).into()
            };
            container(row![icon_el, title_col].spacing(8).align_y(iced::Center))
                .width(Length::Fill)
                .padding([6, 12])
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(bg_color)),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: row_radius.into(),
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
                    .size(title_size)
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

    let maybe_error: Option<Element<'_, Message>> = state.error.as_ref().map(|msg| {
        container(
            text(msg.as_str())
                .size(12.0)
                .color(Color::from_rgba8(255, 80, 80, 1.0)),
        )
        .width(Length::Fill)
        .padding([4, 12])
        .into()
    });

    let mut content_children: Vec<Element<'_, Message>> =
        vec![search_bar.into(), results_list.into()];
    if let Some(err) = maybe_error {
        content_children.push(err);
    }
    let content = column(content_children)
        .spacing(8)
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill);

    let bg_color = rgba(&cfg.background_rgba);
    let border_width = cfg.border_width;
    let border_radius = cfg.border_radius;

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(bg_color)),
            border: Border {
                color: border_color,
                width: border_width,
                radius: border_radius.into(),
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

pub fn run(
    engine: Arc<dyn SearchEngine>,
    launcher: Arc<dyn AppLauncher>,
    window_cfg: &k_launcher_config::WindowCfg,
    appearance_cfg: AppearanceCfg,
) -> iced::Result {
    let wc = WindowConfig::from_cfg(window_cfg);
    iced::application(
        move || {
            let app = KLauncherApp::new(engine.clone(), launcher.clone(), appearance_cfg.clone());
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
