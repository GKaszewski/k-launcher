use std::sync::Arc;

use k_launcher_config::AppearanceCfg;
use k_launcher_domain::AppLauncher;
use k_launcher_domain::{LaunchAction, SearchResult};
use k_launcher_kernel::Kernel;

pub struct LauncherState {
    query: String,
    results: Vec<SearchResult>,
    selected: usize,
    engine: Option<Arc<Kernel>>,
    launcher: Arc<dyn AppLauncher>,
    cfg: AppearanceCfg,
    debounce_ms: u64,
    search_epoch: u64,
    error: Option<String>,
}

pub enum Action {
    QueryChanged(String),
    MoveDown,
    MoveUp,
    LaunchSelected,
    Exit,
    EngineReady(Arc<Kernel>),
    EngineInitFailed(String),
    ResultsReady {
        epoch: u64,
        results: Vec<SearchResult>,
    },
}

pub enum Effect {
    SearchAfterDebounce {
        query: String,
        debounce_ms: u64,
        epoch: u64,
    },
    LaunchAndExit(LaunchAction),
    Exit,
    TriggerSearch(String),
    None,
}

impl LauncherState {
    pub fn new(launcher: Arc<dyn AppLauncher>, cfg: AppearanceCfg, debounce_ms: u64) -> Self {
        Self {
            query: String::new(),
            results: vec![],
            selected: 0,
            engine: None,
            launcher,
            cfg,
            debounce_ms,
            search_epoch: 0,
            error: None,
        }
    }

    pub fn handle(&mut self, action: Action) -> Effect {
        match action {
            Action::QueryChanged(q) => {
                self.query = q;
                self.selected = 0;
                self.error = None;

                let Some(_engine) = &self.engine else {
                    return Effect::None;
                };

                self.search_epoch += 1;
                Effect::SearchAfterDebounce {
                    query: self.query.clone(),
                    debounce_ms: self.debounce_ms,
                    epoch: self.search_epoch,
                }
            }

            Action::MoveDown => {
                let len = self.results.len();
                if len > 0 {
                    self.selected = (self.selected + 1).min(len - 1);
                }
                Effect::None
            }

            Action::MoveUp => {
                self.selected = self.selected.saturating_sub(1);
                Effect::None
            }

            Action::LaunchSelected => {
                if let Some(result) = self.results.get(self.selected) {
                    if let Some(engine) = &self.engine {
                        engine.on_selected(&result.id);
                    }
                    let action = result.action.clone();
                    self.shutdown_engine();
                    return Effect::LaunchAndExit(action);
                }
                self.shutdown_engine();
                Effect::Exit
            }

            Action::Exit => {
                self.shutdown_engine();
                Effect::Exit
            }

            Action::EngineReady(kernel) => {
                self.engine = Some(kernel);
                Effect::TriggerSearch(self.query.clone())
            }

            Action::EngineInitFailed(msg) => {
                self.error = Some(msg);
                Effect::None
            }

            Action::ResultsReady { epoch, results } => {
                if epoch == self.search_epoch {
                    self.results = results;
                }
                Effect::None
            }
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn results(&self) -> &[SearchResult] {
        &self.results
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn cfg(&self) -> &AppearanceCfg {
        &self.cfg
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn is_loading(&self) -> bool {
        self.engine.is_none() && self.error.is_none()
    }

    pub fn engine(&self) -> Option<&Arc<Kernel>> {
        self.engine.as_ref()
    }

    pub fn launcher(&self) -> &Arc<dyn AppLauncher> {
        &self.launcher
    }

    pub fn search_epoch(&self) -> u64 {
        self.search_epoch
    }

    fn shutdown_engine(&self) {
        if let Some(engine) = &self.engine {
            engine.shutdown();
        }
    }
}
