use std::sync::Arc;

use iced::{
    Element, Length,
    widget::{Space, column, container, image, row, scrollable, svg, text, text_input},
};

use k_launcher_config::AppearanceCfg;
use k_launcher_domain::SearchResult;

use crate::app::{INPUT_ID, KLauncherApp, Message};
use crate::style;

pub(crate) fn view(state: &KLauncherApp) -> Element<'_, Message> {
    let cfg = state.inner.cfg();

    let mut content_children: Vec<Element<'_, Message>> =
        vec![search_bar(cfg, state.inner.query()), results_section(state)];
    if let Some(err) = state.inner.error() {
        content_children.push(error_bar(err, cfg));
    }

    let content = column(content_children)
        .spacing(style::CONTENT_SPACING)
        .padding(style::CONTENT_PADDING)
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::outer_container_style(
            style::rgba(&cfg.background_rgba),
            style::rgba(&cfg.border_rgba),
            cfg.border_width,
            cfg.border_radius,
        ))
        .into()
}

fn search_bar<'a>(cfg: &'a AppearanceCfg, query: &'a str) -> Element<'a, Message> {
    text_input(&cfg.placeholder, query)
        .id(INPUT_ID.clone())
        .on_input(Message::QueryChanged)
        .padding(style::CONTENT_PADDING)
        .size(cfg.search_font_size)
        .style(style::search_input_style)
        .into()
}

fn results_section<'a>(state: &'a KLauncherApp) -> Element<'a, Message> {
    if state.inner.is_loading() {
        loading_state(state.inner.cfg())
    } else if state.inner.results().is_empty() && !state.inner.query().is_empty() {
        empty_state(state.inner.cfg())
    } else {
        result_list(state)
    }
}

fn loading_state(cfg: &AppearanceCfg) -> Element<'static, Message> {
    container(
        text("Loading...")
            .size(cfg.title_size)
            .color(style::rgba(&cfg.no_results_rgba)),
    )
    .width(Length::Fill)
    .align_x(iced::Center)
    .padding(style::EMPTY_STATE_PADDING)
    .into()
}

fn empty_state(cfg: &AppearanceCfg) -> Element<'static, Message> {
    scrollable(
        container(
            text("No results")
                .size(cfg.title_size)
                .color(style::rgba(&cfg.no_results_rgba)),
        )
        .width(Length::Fill)
        .align_x(iced::Center)
        .padding(style::EMPTY_STATE_PADDING),
    )
    .height(Length::Fill)
    .into()
}

fn result_list<'a>(state: &'a KLauncherApp) -> Element<'a, Message> {
    let cfg = state.inner.cfg();
    let selected = state.inner.selected();
    let rows: Vec<Element<'_, Message>> = state
        .inner
        .results()
        .iter()
        .enumerate()
        .map(|(i, result)| result_row(result, i == selected, cfg))
        .collect();

    scrollable(column(rows).spacing(style::ROW_SPACING).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

fn result_row<'a>(
    result: &'a SearchResult,
    is_selected: bool,
    cfg: &AppearanceCfg,
) -> Element<'a, Message> {
    let bg_color = if is_selected {
        style::rgba(&cfg.selected_row_rgba)
    } else {
        style::rgba(&cfg.unselected_row_rgba)
    };

    container(
        row![result_icon(&result.icon, cfg), title_column(result, cfg)]
            .spacing(style::CONTENT_SPACING)
            .align_y(iced::Center),
    )
    .width(Length::Fill)
    .padding(style::ROW_PADDING)
    .style(style::result_row_style(bg_color, cfg.row_radius))
    .into()
}

fn result_icon<'a>(icon_path: &'a Option<Arc<str>>, cfg: &AppearanceCfg) -> Element<'a, Message> {
    let size = cfg.icon_size;
    match icon_path {
        Some(p) if p.ends_with(".svg") => svg(svg::Handle::from_path(p.as_ref()))
            .width(size)
            .height(size)
            .into(),
        Some(p) => image(image::Handle::from_path(p.as_ref()))
            .width(size)
            .height(size)
            .into(),
        None => Space::new().width(size).height(size).into(),
    }
}

fn title_column<'a>(result: &'a SearchResult, cfg: &AppearanceCfg) -> Element<'a, Message> {
    if let Some(desc) = &result.description {
        column![
            text(result.title.as_str()).size(cfg.title_size),
            text(desc.as_ref())
                .size(cfg.desc_size)
                .color(style::rgba(&cfg.description_rgba)),
        ]
        .into()
    } else {
        text(result.title.as_str()).size(cfg.title_size).into()
    }
}

fn error_bar<'a>(msg: &'a str, cfg: &AppearanceCfg) -> Element<'a, Message> {
    container(
        text(msg)
            .size(style::ERROR_FONT_SIZE)
            .color(style::rgba(&cfg.error_rgba)),
    )
    .width(Length::Fill)
    .padding(style::ERROR_PADDING)
    .into()
}
