use egui::Key;

pub enum InputAction {
    Close,
    LaunchSelected,
    MoveDown,
    MoveUp,
    None,
}

pub fn process_input(ctx: &egui::Context) -> InputAction {
    ctx.input(|i| {
        if i.key_pressed(Key::Escape) {
            InputAction::Close
        } else if i.key_pressed(Key::Enter) {
            InputAction::LaunchSelected
        } else if i.key_pressed(Key::ArrowDown) {
            InputAction::MoveDown
        } else if i.key_pressed(Key::ArrowUp) {
            InputAction::MoveUp
        } else {
            InputAction::None
        }
    })
}
