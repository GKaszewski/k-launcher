use std::sync::Arc;

use iced::{Size, Subscription, Task, event, keyboard::Event as KeyEvent, window};

use k_launcher_config::AppearanceCfg;
use k_launcher_domain::AppLauncher;
use k_launcher_domain::SearchResult;
use k_launcher_kernel::Kernel;
use k_launcher_ui_core::LauncherState;

pub(crate) static INPUT_ID: std::sync::LazyLock<iced::widget::Id> =
    std::sync::LazyLock::new(|| iced::widget::Id::new("search"));

#[derive(Clone)]
pub(crate) struct EngineHandle(pub(crate) Arc<Kernel>);

impl std::fmt::Debug for EngineHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EngineHandle")
    }
}

pub(crate) struct KLauncherApp {
    pub(crate) inner: LauncherState,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    QueryChanged(String),
    ResultsReady {
        epoch: u64,
        results: Arc<Vec<SearchResult>>,
    },
    KeyPressed(KeyEvent),
    EngineReady(EngineHandle),
    EngineInitFailed(String),
}

fn subscription(_state: &KLauncherApp) -> Subscription<Message> {
    event::listen_with(|ev, _status, _id| match ev {
        iced::Event::Keyboard(ke) => Some(Message::KeyPressed(ke)),
        _ => None,
    })
}

pub fn run(
    engine_factory: Arc<dyn Fn() -> Arc<Kernel> + Send + Sync>,
    launcher: Arc<dyn AppLauncher>,
    window_cfg: &k_launcher_config::WindowCfg,
    appearance_cfg: AppearanceCfg,
    debounce_ms: u64,
) -> iced::Result {
    iced::application(
        move || {
            let inner = LauncherState::new(launcher.clone(), appearance_cfg.clone(), debounce_ms);
            let app = KLauncherApp { inner };
            let focus = iced::widget::operation::focus(INPUT_ID.clone());
            let ef = engine_factory.clone();
            let init = Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || ef())
                        .await
                        .map_err(|e| format!("Engine init failed: {e}"))
                },
                |result| match result {
                    Ok(e) => Message::EngineReady(EngineHandle(e)),
                    Err(msg) => Message::EngineInitFailed(msg),
                },
            );
            (app, Task::batch([focus, init]))
        },
        crate::update::update,
        crate::view::view,
    )
    .title(k_launcher_domain::constants::APP_TITLE)
    .subscription(subscription)
    .window(window::Settings {
        size: Size::new(window_cfg.width, window_cfg.height),
        position: window::Position::Centered,
        decorations: window_cfg.decorations,
        transparent: window_cfg.transparent,
        resizable: window_cfg.resizable,
        ..Default::default()
    })
    .run()
}
