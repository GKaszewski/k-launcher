use egui::Ui;
use k_launcher_config::AppearanceCfg;
use k_launcher_domain::SearchResult;

use crate::style::{self, ROW_SPACING, SEARCH_BAR_HEIGHT, to_color32};

pub fn render_search_bar(ui: &mut Ui, query: &mut String, cfg: &AppearanceCfg) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), SEARCH_BAR_HEIGHT],
        egui::TextEdit::singleline(query)
            .hint_text(&cfg.placeholder)
            .font(egui::TextStyle::Heading),
    )
}

pub fn render_loading_state(ui: &mut Ui, cfg: &AppearanceCfg) {
    ui.add_space(20.0);
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.colored_label(to_color32(&cfg.no_results_rgba), "Loading...");
    });
}

pub fn render_empty_state(ui: &mut Ui, cfg: &AppearanceCfg) {
    ui.add_space(20.0);
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.colored_label(to_color32(&cfg.no_results_rgba), "No results");
    });
}

pub fn render_result_list(
    ui: &mut Ui,
    results: &[SearchResult],
    selected: usize,
    cfg: &AppearanceCfg,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        for (i, result) in results.iter().enumerate() {
            render_result_row(ui, result, i == selected, cfg);
            ui.add_space(ROW_SPACING);
        }
    });
}

fn render_result_row(ui: &mut Ui, result: &SearchResult, is_selected: bool, cfg: &AppearanceCfg) {
    style::result_row_frame(is_selected, cfg).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.label(result.title.as_str());
                if let Some(desc) = &result.description {
                    ui.colored_label(to_color32(&cfg.description_rgba), desc.as_ref());
                }
            });
        });
    });
}
