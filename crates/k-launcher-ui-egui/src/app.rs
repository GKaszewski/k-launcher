use std::sync::{Arc, mpsc};

use egui::{Color32, Key, ViewportCommand};
use k_launcher_kernel::{AppLauncher, SearchEngine, SearchResult};
use k_launcher_os_bridge::WindowConfig;

const BG: Color32 = Color32::from_rgba_premultiplied(20, 20, 30, 230);
const BORDER_COLOR: Color32 = Color32::from_rgb(229, 125, 33);
const SELECTED_BG: Color32 = Color32::from_rgba_premultiplied(0, 100, 140, 180);
const DIM_TEXT: Color32 = Color32::from_rgb(180, 185, 200);

pub struct KLauncherApp {
    engine: Arc<dyn SearchEngine>,
    launcher: Arc<dyn AppLauncher>,
    query: String,
    results: Vec<SearchResult>,
    selected: usize,
    rt: tokio::runtime::Handle,
    result_tx: mpsc::SyncSender<Vec<SearchResult>>,
    result_rx: mpsc::Receiver<Vec<SearchResult>>,
}

impl KLauncherApp {
    fn new(
        engine: Arc<dyn SearchEngine>,
        launcher: Arc<dyn AppLauncher>,
        rt: tokio::runtime::Handle,
    ) -> Self {
        let (result_tx, result_rx) = mpsc::sync_channel(4);
        Self {
            engine,
            launcher,
            query: String::new(),
            results: vec![],
            selected: 0,
            rt,
            result_tx,
            result_rx,
        }
    }

    fn trigger_search(&self, query: String) {
        let engine = self.engine.clone();
        let tx = self.result_tx.clone();
        self.rt.spawn(async move {
            let results = engine.search(&query).await;
            let _ = tx.send(results);
        });
    }
}

impl eframe::App for KLauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(results) = self.result_rx.try_recv() {
            self.results = results;
        }

        let mut close = false;
        let mut launch_selected = false;

        ctx.input(|i| {
            if i.key_pressed(Key::Escape) {
                close = true;
            }
            if i.key_pressed(Key::Enter) {
                launch_selected = true;
            }
            if i.key_pressed(Key::ArrowDown) {
                let len = self.results.len();
                if len > 0 {
                    self.selected = (self.selected + 1).min(len - 1);
                }
            }
            if i.key_pressed(Key::ArrowUp) && self.selected > 0 {
                self.selected -= 1;
            }
        });

        if close {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        if launch_selected {
            if let Some(result) = self.results.get(self.selected) {
                if let Some(on_select) = &result.on_select {
                    on_select();
                }
                self.launcher.execute(&result.action);
            }
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        let frame = egui::Frame::new()
            .fill(BG)
            .stroke(egui::Stroke::new(1.0, BORDER_COLOR))
            .inner_margin(egui::Margin::same(12))
            .corner_radius(egui::CornerRadius::same(8));

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let response = ui.add_sized(
                [ui.available_width(), 36.0],
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("Search...")
                    .font(egui::TextStyle::Heading),
            );

            if response.changed() {
                self.selected = 0;
                self.trigger_search(self.query.clone());
            }

            response.request_focus();

            ui.add_space(8.0);

            if self.results.is_empty() && !self.query.is_empty() {
                ui.add_space(20.0);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.colored_label(DIM_TEXT, "No results");
                });
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_width(ui.available_width());
                for (i, result) in self.results.iter().enumerate() {
                    let is_selected = i == self.selected;
                    let bg = if is_selected { SELECTED_BG } else { Color32::TRANSPARENT };

                    let row_frame = egui::Frame::new()
                        .fill(bg)
                        .inner_margin(egui::Margin { left: 8, right: 8, top: 6, bottom: 6 })
                        .corner_radius(egui::CornerRadius::same(4));

                    row_frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.label(result.title.as_str());
                                if let Some(desc) = &result.description {
                                    ui.colored_label(DIM_TEXT, desc);
                                }
                            });
                        });
                    });

                    ui.add_space(2.0);
                }
            });
        });
    }
}

pub fn run(
    engine: Arc<dyn SearchEngine>,
    launcher: Arc<dyn AppLauncher>,
) -> Result<(), eframe::Error> {
    let wc = WindowConfig::from_cfg(&k_launcher_config::WindowCfg::default());
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let handle = rt.handle().clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([wc.width, wc.height])
            .with_decorations(wc.decorations)
            .with_transparent(wc.transparent)
            .with_resizable(wc.resizable)
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "K-Launcher",
        options,
        Box::new(move |_cc| Ok(Box::new(KLauncherApp::new(engine, launcher, handle)))),
    )
}
