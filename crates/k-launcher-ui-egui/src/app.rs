use std::sync::{Arc, mpsc};

use egui::ViewportCommand;
use k_launcher_config::AppearanceCfg;
use k_launcher_domain::AppLauncher;
use k_launcher_domain::SearchResult;
use k_launcher_kernel::Kernel;
use k_launcher_ui_core::{Action, Effect, LauncherState};

use crate::input::{InputAction, process_input};
use crate::render;
use crate::style;

pub struct KLauncherApp {
    pub(crate) inner: LauncherState,
    rt: tokio::runtime::Handle,
    result_tx: mpsc::SyncSender<Vec<SearchResult>>,
    pub(crate) result_rx: mpsc::Receiver<Vec<SearchResult>>,
}

impl KLauncherApp {
    fn new(
        engine: Arc<Kernel>,
        launcher: Arc<dyn AppLauncher>,
        rt: tokio::runtime::Handle,
        cfg: AppearanceCfg,
    ) -> Self {
        const RESULT_CHANNEL_CAPACITY: usize = 4;
        let (result_tx, result_rx) = mpsc::sync_channel(RESULT_CHANNEL_CAPACITY);
        let mut inner = LauncherState::new(launcher, cfg, 0);

        let effect = inner.handle(Action::EngineReady(engine));

        let app = Self {
            inner,
            rt,
            result_tx,
            result_rx,
        };

        app.execute_effect(effect);
        app
    }

    fn trigger_search(&self, query: String) {
        let Some(engine) = self.inner.engine().cloned() else {
            return;
        };
        let tx = self.result_tx.clone();
        self.rt.spawn(async move {
            let results = engine.search(&query).await;
            if let Err(e) = tx.send(results) {
                tracing::warn!("search result channel closed: {e}");
            }
        });
    }

    fn poll_search_results(&mut self) {
        if let Ok(results) = self.result_rx.try_recv() {
            self.inner.handle(Action::ResultsReady {
                epoch: self.inner.search_epoch(),
                results,
            });
        }
    }

    fn execute_effect(&self, effect: Effect) {
        match effect {
            Effect::TriggerSearch(q) => self.trigger_search(q),
            Effect::SearchAfterDebounce { query, .. } => self.trigger_search(query),
            _ => {}
        }
    }

    fn handle_action(&mut self, action: Action, ctx: &egui::Context) {
        let effect = self.inner.handle(action);
        match effect {
            Effect::LaunchAndExit(action) => {
                self.inner.launcher().execute(&action);
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            Effect::Exit => {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            other => self.execute_effect(other),
        }
    }

    fn render_panel(&mut self, ctx: &egui::Context) {
        let cfg = self.inner.cfg().clone();
        egui::CentralPanel::default()
            .frame(style::outer_frame(&cfg))
            .show(ctx, |ui| {
                let query = self.inner.query().to_string();
                let mut query_buf = query;
                let response = render::render_search_bar(ui, &mut query_buf, &cfg);

                if response.changed() {
                    self.handle_action(Action::QueryChanged(query_buf), ctx);
                }

                response.request_focus();
                ui.add_space(8.0);

                if self.inner.is_loading() {
                    render::render_loading_state(ui, &cfg);
                    return;
                }

                if self.inner.results().is_empty() && !self.inner.query().is_empty() {
                    render::render_empty_state(ui, &cfg);
                    return;
                }

                render::render_result_list(ui, self.inner.results(), self.inner.selected(), &cfg);
            });
    }
}

impl eframe::App for KLauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_search_results();

        match process_input(ctx) {
            InputAction::Close => {
                self.handle_action(Action::Exit, ctx);
                return;
            }
            InputAction::LaunchSelected => {
                self.handle_action(Action::LaunchSelected, ctx);
                return;
            }
            InputAction::MoveDown => {
                self.inner.handle(Action::MoveDown);
            }
            InputAction::MoveUp => {
                self.inner.handle(Action::MoveUp);
            }
            InputAction::None => {}
        }

        self.render_panel(ctx);
    }
}

pub fn run(
    engine: Arc<Kernel>,
    launcher: Arc<dyn AppLauncher>,
    window_cfg: &k_launcher_config::WindowCfg,
    appearance_cfg: AppearanceCfg,
) -> Result<(), eframe::Error> {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let handle = rt.handle().clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([window_cfg.width, window_cfg.height])
            .with_decorations(window_cfg.decorations)
            .with_transparent(window_cfg.transparent)
            .with_resizable(window_cfg.resizable)
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        k_launcher_domain::constants::APP_TITLE,
        options,
        Box::new(move |_cc| {
            Ok(Box::new(KLauncherApp::new(
                engine,
                launcher,
                handle,
                appearance_cfg,
            )))
        }),
    )
}
