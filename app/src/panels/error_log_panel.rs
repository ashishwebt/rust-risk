use crate::panel_frame;
use crate::state::AppState;
use eframe::egui;

/// A bottom bar that displays the last N errors from the rolling error log.
/// Errors are shown newest-first with a timestamp. The user can clear the log.
pub fn error_log_panel(ui: &mut egui::Ui, state: &mut AppState) {
    panel_frame()
        .fill(egui::Color32::from_rgb(22, 14, 14)) // dark-red tint to distinguish errors
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(80, 30, 30),
        ))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚠  Error Log")
                        .size(13.0)
                        .color(egui::Color32::from_rgb(255, 100, 100))
                        .strong(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(
                            egui::RichText::new("Clear")
                                .color(egui::Color32::from_rgb(160, 100, 100)),
                        )
                        .clicked()
                    {
                        state.error_log.clear();
                        state.last_error = None;
                    }

                    ui.label(
                        egui::RichText::new(format!("{} entries", state.error_log.len()))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(120, 100, 100)),
                    );
                });
            });

            ui.add_space(4.0);
            ui.separator();

            if state.error_log.is_empty() {
                ui.label(
                    egui::RichText::new("No errors recorded.")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(100, 110, 100)),
                );
                return;
            }

            // Show errors newest-first.
            egui::ScrollArea::vertical()
                .id_salt("error_log_scroll")
                .max_height(80.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in state.error_log.iter().rev() {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&entry.timestamp)
                                    .size(10.0)
                                    .monospace()
                                    .color(egui::Color32::from_rgb(140, 100, 100)),
                            );
                            ui.label(
                                egui::RichText::new(&entry.message)
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(220, 150, 150)),
                            );
                        });
                    }
                });
        });
}
