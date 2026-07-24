use std::sync::Arc;

use iced::{
    Task,
    keyboard::{Event as KeyEvent, Key, key::Named},
};

use k_launcher_ui_core::{Action, Effect};

use crate::app::{KLauncherApp, Message};

pub(crate) fn update(state: &mut KLauncherApp, message: Message) -> Task<Message> {
    let action = match message {
        Message::QueryChanged(q) => Action::QueryChanged(q),
        Message::ResultsReady { epoch, results } => {
            let results = Arc::try_unwrap(results).unwrap_or_else(|arc| (*arc).clone());
            Action::ResultsReady { epoch, results }
        }
        Message::EngineInitFailed(msg) => Action::EngineInitFailed(msg),
        Message::EngineReady(handle) => Action::EngineReady(handle.0),
        Message::KeyPressed(event) => match map_key_event(event) {
            Some(a) => a,
            None => return Task::none(),
        },
    };

    let effect = state.inner.handle(action);
    execute_effect(state, effect)
}

fn map_key_event(event: KeyEvent) -> Option<Action> {
    let key = match event {
        KeyEvent::KeyPressed { key, .. } => key,
        _ => return None,
    };
    let Key::Named(named) = key else {
        return None;
    };
    match named {
        Named::Escape => Some(Action::Exit),
        Named::ArrowDown => Some(Action::MoveDown),
        Named::ArrowUp => Some(Action::MoveUp),
        Named::Enter => Some(Action::LaunchSelected),
        _ => None,
    }
}

fn execute_effect(state: &KLauncherApp, effect: Effect) -> Task<Message> {
    match effect {
        Effect::SearchAfterDebounce {
            query,
            debounce_ms,
            epoch,
        } => {
            let Some(engine) = state.inner.engine().cloned() else {
                return Task::none();
            };
            Task::perform(
                async move {
                    tokio::time::sleep(std::time::Duration::from_millis(debounce_ms)).await;
                    (epoch, engine.search(&query).await)
                },
                |(epoch, results)| Message::ResultsReady {
                    epoch,
                    results: Arc::new(results),
                },
            )
        }
        Effect::LaunchAndExit(action) => {
            state.inner.launcher().execute(&action);
            iced::exit()
        }
        Effect::Exit => iced::exit(),
        Effect::TriggerSearch(q) => Task::done(Message::QueryChanged(q)),
        Effect::None => Task::none(),
    }
}
