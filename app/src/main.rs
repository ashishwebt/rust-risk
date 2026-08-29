mod panels;
mod state;

use data_feed::FeedStatus;
use eframe::egui;
use state::{AppState, SourceChoice};

struct DashboardApp {
    state: AppState,
}

impl Default for DashboardApp {
    fn default() -> Self {
        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for DashboardApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Non-UI logic belongs here in eframe 0.36.
        // Drain the normalized feed channel.
        self.state.pump_feed();

        // Keep refreshing for real-time market data.
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Since eframe 0.36 gives us a root Ui instead of the old
        // Context-based update() API, we build the dashboard inside it.

        egui::Frame::default()
            .show(ui, |ui| {
                // =========================
                // Toolbar
                // =========================

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.heading("Market Risk Dashboard");
                    ui.separator();

                    ui.label("Data source:");

                    if ui
                        .selectable_label(
                            self.state.source_choice == SourceChoice::Simulated,
                            "Simulated",
                        )
                        .clicked()
                    {
                        self.state.switch_source(SourceChoice::Simulated);
                    }

                    if ui
                        .selectable_label(
                            self.state.source_choice == SourceChoice::Yahoo,
                            "Yahoo Finance",
                        )
                        .clicked()
                    {
                        self.state.switch_source(SourceChoice::Yahoo);
                    }

                    ui.separator();

                    let (dot, text) = match &self.state.feed_status {
                        FeedStatus::Connecting => {
                            (egui::Color32::YELLOW, "connecting...".to_string())
                        }

                        FeedStatus::Connected => {
                            (egui::Color32::GREEN, "connected".to_string())
                        }

                        FeedStatus::Disconnected(reason) => {
                            (
                                egui::Color32::GRAY,
                                format!("disconnected: {reason}"),
                            )
                        }

                        FeedStatus::Error(reason) => {
                            (
                                egui::Color32::RED,
                                format!("error: {reason}"),
                            )
                        }
                    };

                    ui.colored_label(dot, "●");
                    ui.label(text);
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // =========================
                // Dashboard grid
                // =========================

                ui.columns(2, |columns| {
                    // LEFT COLUMN
                    egui::ScrollArea::vertical()
                        .id_salt("col_left")
                        .show(&mut columns[0], |ui| {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                panels::positions::positions_panel(
                                    ui,
                                    &self.state,
                                );
                            });

                            ui.add_space(10.0);

                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                panels::var_panel::var_panel(
                                    ui,
                                    &mut self.state,
                                );
                            });
                        });

                    // RIGHT COLUMN
                    egui::ScrollArea::vertical()
                        .id_salt("col_right")
                        .show(&mut columns[1], |ui| {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                panels::vol_surface_panel::vol_surface_panel(
                                    ui,
                                    &self.state,
                                );
                            });

                            ui.add_space(10.0);

                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                panels::stress_panel::stress_panel(
                                    ui,
                                    &self.state,
                                );
                            });
                        });
                });
            });
    }
}
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Market Risk Dashboard",
        options,
        Box::new(|_cc| Ok(Box::new(DashboardApp::default()) as Box<dyn eframe::App>)),
    )
}