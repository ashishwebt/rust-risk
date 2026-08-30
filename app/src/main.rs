mod db;
mod panels;
mod provider;
mod state;

use data_feed::FeedStatus;
use eframe::egui;
use egui::epaint::CornerRadius;
use egui_extras; // StripBuilder for non-overlapping columns
use state::{AppState, SourceChoice};
use tracing::info;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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

/// Initialize the tracing subscriber.
///
/// - Writes structured JSON logs to `logs/dashboard.log` (rolling daily).
/// - Writes human-readable logs to stderr (controlled by `RUST_LOG`; defaults to `info`).
fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    // Create logs directory next to the binary if it doesn't exist.
    let _ = std::fs::create_dir_all("logs");

    // Rolling file appender (new file each day: logs/dashboard.YYYY-MM-DD)
    let file_appender = rolling::daily("logs", "dashboard.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // JSON layer → file
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_ansi(false);

    // Pretty layer → stderr
    let stderr_layer = fmt::layer()
        .pretty()
        .with_writer(std::io::stderr)
        .with_ansi(true);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stderr_layer)
        .init();

    guard
}

/// Apply a dark finance-terminal theme.
fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Background / panel shades
    visuals.panel_fill = egui::Color32::from_rgb(14, 17, 23);
    visuals.window_fill = egui::Color32::from_rgb(14, 17, 23);
    visuals.faint_bg_color = egui::Color32::from_rgb(20, 24, 33);
    visuals.extreme_bg_color = egui::Color32::from_rgb(10, 12, 18);

    // Widget backgrounds
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(22, 27, 38);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(28, 34, 48);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(38, 46, 64);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(48, 100, 180);

    // Borders
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 50, 68));
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 62, 85));

    // Text
    visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(190, 200, 220));
    visuals.override_text_color = Some(egui::Color32::from_rgb(200, 210, 230));

    // Selection / accent
    visuals.selection.bg_fill = egui::Color32::from_rgb(30, 80, 160);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 150, 255));

    // Window corner radius
    visuals.window_corner_radius = CornerRadius::same(6);

    ctx.set_visuals(visuals);

    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
    });
}

impl eframe::App for DashboardApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.pump_feed();
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::default()
            .inner_margin(8i8)
            .show(ui, |ui| {
                // =========================
                // Toolbar
                // =========================
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new("⬡  Market Risk Dashboard")
                                .size(18.0)
                                .color(egui::Color32::from_rgb(120, 180, 255))
                                .strong(),
                        )
                        .wrap(),
                    );

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(
                        egui::RichText::new("Data source:")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(140, 155, 175)),
                    );

                    for (label, choice) in [
                        ("Simulated", SourceChoice::Simulated),
                        ("Yahoo Finance", SourceChoice::Yahoo),
                    ] {
                        let selected = self.state.source_choice == choice;
                        let text = if selected {
                            egui::RichText::new(label)
                                .color(egui::Color32::from_rgb(100, 200, 255))
                                .strong()
                        } else {
                            egui::RichText::new(label)
                                .color(egui::Color32::from_rgb(160, 175, 195))
                        };
                        if ui.selectable_label(selected, text).clicked() {
                            self.state.switch_source(choice);
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Status badge
                    let (status_color, status_text) = match &self.state.feed_status {
                        FeedStatus::Connecting => (
                            egui::Color32::from_rgb(255, 200, 50),
                            "● Connecting…".to_string(),
                        ),
                        FeedStatus::Connected => (
                            egui::Color32::from_rgb(50, 220, 120),
                            "● Connected".to_string(),
                        ),
                        FeedStatus::Disconnected(r) => (
                            egui::Color32::from_rgb(140, 145, 155),
                            format!("● Disconnected: {r}"),
                        ),
                        FeedStatus::Error(r) => (
                            egui::Color32::from_rgb(255, 80, 80),
                            format!("● Error: {r}"),
                        ),
                    };
                    ui.label(
                        egui::RichText::new(status_text)
                            .size(12.0)
                            .color(status_color),
                    );
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(10.0);

                // =========================
                // Dashboard grid
                // =========================
                egui_extras::StripBuilder::new(ui)
                    .size(egui_extras::Size::remainder()) // main rows
                    .size(egui_extras::Size::exact(8.0))  // gutter
                    .size(egui_extras::Size::exact(140.0)) // error log panel height
                    .vertical(|mut strip| {
                        // TOP: two-column layout
                        strip.cell(|ui| {
                            egui_extras::StripBuilder::new(ui)
                                .size(egui_extras::Size::remainder())
                                .size(egui_extras::Size::exact(8.0))
                                .size(egui_extras::Size::remainder())
                                .horizontal(|mut strip| {
                                    // LEFT COLUMN
                                    strip.cell(|ui| {
                                        egui::ScrollArea::vertical()
                                            .id_salt("col_left")
                                            .show(ui, |ui| {
                                                panel_frame().show(ui, |ui| {
                                                    panels::positions::positions_panel(
                                                        ui,
                                                        &mut self.state,
                                                    );
                                                    panels::symbol_form_panel::symbol_form_panel(
                                                        ui,
                                                        &mut self.state,
                                                    );
                                                });
                                                ui.add_space(12.0);
                                                panel_frame().show(ui, |ui| {
                                                    panels::var_panel::var_panel(
                                                        ui,
                                                        &mut self.state,
                                                    );
                                                });
                                            });
                                    });

                                    // GUTTER
                                    strip.empty();

                                    // RIGHT COLUMN
                                    strip.cell(|ui| {
                                        egui::ScrollArea::vertical()
                                            .id_salt("col_right")
                                            .show(ui, |ui| {
                                                panel_frame().show(ui, |ui| {
                                                    panels::vol_surface_panel::vol_surface_panel(
                                                        ui,
                                                        &self.state,
                                                    );
                                                });
                                                ui.add_space(12.0);
                                                panel_frame().show(ui, |ui| {
                                                    panels::stress_panel::stress_panel(
                                                        ui,
                                                        &self.state,
                                                    );
                                                });
                                            });
                                    });
                                });
                        });

                        // GUTTER
                        strip.empty();

                        // BOTTOM: Error log panel
                        strip.cell(|ui| {
                            panels::error_log_panel::error_log_panel(ui, &mut self.state);
                        });
                    });
            });
    }
}

/// Consistent panel frame used for all dashboard cards.
pub fn panel_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(18, 22, 32))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(38, 48, 68)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(12i8)
}

fn main() -> eframe::Result<()> {
    // Keep the guard alive for the duration of main so the log file is flushed.
    let _tracing_guard = init_tracing();

    info!("Market Risk Dashboard starting up");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 860.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Market Risk Dashboard",
        options,
        Box::new(|cc| {
            setup_visuals(&cc.egui_ctx);
            Ok(Box::new(DashboardApp::default()) as Box<dyn eframe::App>)
        }),
    )
}
