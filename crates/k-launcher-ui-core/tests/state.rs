use std::sync::Arc;

use k_launcher_config::AppearanceCfg;
use k_launcher_domain::AppLauncher;
use k_launcher_domain::*;
use k_launcher_kernel::Kernel;
use k_launcher_ui_core::{Action, Effect, LauncherState};

struct NoopLauncher;

impl AppLauncher for NoopLauncher {
    fn execute(&self, _action: &LaunchAction) {}
}

fn make_state() -> LauncherState {
    LauncherState::new(Arc::new(NoopLauncher), AppearanceCfg::default(), 50)
}

fn make_result(id: &str) -> SearchResult {
    SearchResult {
        id: ResultId::new(id),
        title: ResultTitle::new(id),
        description: None,
        icon: None,
        score: Score::new(100),
        action: LaunchAction::CopyToClipboard(id.to_string()),
    }
}

fn make_state_with_engine() -> LauncherState {
    let kernel = Arc::new(Kernel::new(vec![], 10));
    let mut state = make_state();
    let _ = state.handle(Action::EngineReady(kernel));
    state
}

#[test]
fn move_down_clamps_to_last_result() {
    let mut state = make_state();
    state.handle(Action::ResultsReady {
        epoch: 0,
        results: vec![make_result("a"), make_result("b"), make_result("c")],
    });

    state.handle(Action::MoveDown);
    state.handle(Action::MoveDown);
    state.handle(Action::MoveDown);
    state.handle(Action::MoveDown);

    assert_eq!(state.selected(), 2);
}

#[test]
fn move_up_does_not_go_below_zero() {
    let mut state = make_state();
    state.handle(Action::ResultsReady {
        epoch: 0,
        results: vec![make_result("a"), make_result("b")],
    });

    state.handle(Action::MoveUp);
    state.handle(Action::MoveUp);

    assert_eq!(state.selected(), 0);
}

#[test]
fn query_changed_resets_selected() {
    let mut state = make_state_with_engine();
    state.handle(Action::ResultsReady {
        epoch: state.search_epoch(),
        results: vec![make_result("a"), make_result("b"), make_result("c")],
    });
    state.handle(Action::MoveDown);
    state.handle(Action::MoveDown);
    assert_eq!(state.selected(), 2);

    state.handle(Action::QueryChanged("new".to_string()));
    assert_eq!(state.selected(), 0);
}

#[test]
fn engine_ready_returns_trigger_search() {
    let mut state = make_state();
    let kernel = Arc::new(Kernel::new(vec![], 10));
    let effect = state.handle(Action::EngineReady(kernel));

    assert!(matches!(effect, Effect::TriggerSearch(_)));
}

#[test]
fn launch_selected_with_no_results_returns_exit() {
    let mut state = make_state_with_engine();
    let effect = state.handle(Action::LaunchSelected);

    assert!(matches!(effect, Effect::Exit));
}

#[test]
fn results_ready_with_wrong_epoch_is_ignored() {
    let mut state = make_state_with_engine();
    let current_epoch = state.search_epoch();

    state.handle(Action::ResultsReady {
        epoch: current_epoch + 999,
        results: vec![make_result("stale")],
    });

    assert!(state.results().is_empty());
}

#[test]
fn query_changed_without_engine_returns_none() {
    let mut state = make_state();
    let effect = state.handle(Action::QueryChanged("hello".to_string()));

    assert!(matches!(effect, Effect::None));
    assert_eq!(state.query(), "hello");
}

#[test]
fn query_changed_with_engine_returns_search_after_debounce() {
    let mut state = make_state_with_engine();
    let effect = state.handle(Action::QueryChanged("test".to_string()));

    match effect {
        Effect::SearchAfterDebounce {
            query,
            debounce_ms,
            epoch,
        } => {
            assert_eq!(query, "test");
            assert_eq!(debounce_ms, 50);
            assert_eq!(epoch, state.search_epoch());
        }
        _ => panic!("expected SearchAfterDebounce"),
    }
}

#[test]
fn launch_selected_with_results_returns_launch_and_exit() {
    let mut state = make_state_with_engine();
    state.handle(Action::ResultsReady {
        epoch: state.search_epoch(),
        results: vec![make_result("app1")],
    });

    let effect = state.handle(Action::LaunchSelected);
    assert!(matches!(effect, Effect::LaunchAndExit(_)));
}

#[test]
fn engine_init_failed_sets_error() {
    let mut state = make_state();
    state.handle(Action::EngineInitFailed("boom".to_string()));

    assert_eq!(state.error(), Some("boom"));
}
