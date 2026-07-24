use iced::{Border, Color, widget::container, widget::text_input};

// ---- layout constants ----
pub(crate) const ROW_PADDING: [u16; 2] = [6, 12];
pub(crate) const ROW_SPACING: f32 = 2.0;
pub(crate) const CONTENT_PADDING: u16 = 12;
pub(crate) const CONTENT_SPACING: f32 = 8.0;
pub(crate) const EMPTY_STATE_PADDING: [u16; 2] = [20, 0];
pub(crate) const ERROR_FONT_SIZE: f32 = 12.0;
pub(crate) const ERROR_PADDING: [u16; 2] = [4, 12];

// ---- helpers ----
pub(crate) fn rgba(c: &k_launcher_config::Rgba) -> Color {
    Color::from_rgba8(c.red_u8(), c.green_u8(), c.blue_u8(), c.alpha())
}

pub(crate) fn search_input_style(
    theme: &iced::Theme,
    _status: text_input::Status,
) -> text_input::Style {
    let mut s = text_input::default(theme, text_input::Status::Active);
    s.border = Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: 0.0.into(),
    };
    s
}

pub(crate) fn result_row_style(
    bg_color: Color,
    row_radius: f32,
) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(iced::Background::Color(bg_color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: row_radius.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn outer_container_style(
    bg: Color,
    border_color: Color,
    width: f32,
    radius: f32,
) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border {
            color: border_color,
            width,
            radius: radius.into(),
        },
        ..Default::default()
    }
}
