use egui::{Color32, CornerRadius, Frame, Margin, Stroke};
use k_launcher_config::AppearanceCfg;

pub const SEARCH_BAR_HEIGHT: f32 = 36.0;
pub const CONTENT_MARGIN: i8 = 12;
pub const ROW_SPACING: f32 = 2.0;

const ROW_PADDING_VERTICAL: i8 = 6;
const ROW_PADDING_HORIZONTAL: i8 = 8;

pub(crate) fn to_color32(c: &k_launcher_config::Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(c.red_u8(), c.green_u8(), c.blue_u8(), c.alpha_byte())
}

pub fn outer_frame(cfg: &AppearanceCfg) -> Frame {
    Frame::new()
        .fill(to_color32(&cfg.background_rgba))
        .stroke(Stroke::new(cfg.border_width, to_color32(&cfg.border_rgba)))
        .inner_margin(Margin::same(CONTENT_MARGIN))
        .corner_radius(CornerRadius::same(cfg.border_radius as u8))
}

pub fn result_row_frame(is_selected: bool, cfg: &AppearanceCfg) -> Frame {
    let bg = if is_selected {
        to_color32(&cfg.selected_row_rgba)
    } else {
        to_color32(&cfg.unselected_row_rgba)
    };

    Frame::new()
        .fill(bg)
        .inner_margin(Margin {
            left: ROW_PADDING_HORIZONTAL,
            right: ROW_PADDING_HORIZONTAL,
            top: ROW_PADDING_VERTICAL,
            bottom: ROW_PADDING_VERTICAL,
        })
        .corner_radius(CornerRadius::same(cfg.row_radius as u8))
}
