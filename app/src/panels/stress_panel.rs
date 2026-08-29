use crate::state::AppState;
use egui::epaint::CornerRadius;
use egui_extras::{Column, TableBuilder};
use super::positions::panel_header;

const DIM: egui::Color32 = egui::Color32::from_rgb(100, 115, 140);
const POSITIVE: egui::Color32 = egui::Color32::from_rgb(60, 210, 120);
const NEGATIVE: egui::Color32 = egui::Color32::from_rgb(240, 90, 90);

pub fn stress_panel(ui: &mut egui::Ui, state: &AppState) {
    panel_header(ui, "⚡  Stress Scenarios");
    ui.add_space(8.0);

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(230.0)) // Scenario name
        .column(Column::auto().at_least(105.0))      // Base Value
        .column(Column::auto().at_least(105.0))      // Stressed Value
        .column(Column::auto().at_least(110.0))      // P&L
        .header(24.0, |mut header| {
            for label in ["Scenario", "Base Value", "Stressed Value", "P&L"] {
                header.col(|ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .color(DIM)
                            .size(11.0)
                            .strong(),
                    );
                });
            }
        })
        .body(|mut body| {
            for result in &state.scenario_results {
                body.row(28.0, |mut row| {
                    row.col(|ui| {
                        ui.label(
                            egui::RichText::new(&result.scenario_name)
                                .color(egui::Color32::from_rgb(200, 210, 235)),
                        );
                    });
                    row.col(|ui| {
                        ui.label(
                            egui::RichText::new(format!("${:.0}", result.base_value))
                                .color(egui::Color32::from_rgb(170, 180, 205)),
                        );
                    });
                    row.col(|ui| {
                        ui.label(
                            egui::RichText::new(format!("${:.0}", result.stressed_value))
                                .color(egui::Color32::from_rgb(200, 210, 230)),
                        );
                    });
                    row.col(|ui| {
                        let (color, sign) = if result.pnl >= 0.0 {
                            (POSITIVE, "+")
                        } else {
                            (NEGATIVE, "")
                        };
                        // Subtle cell background tint
                        let bg = if result.pnl >= 0.0 {
                            egui::Color32::from_rgba_unmultiplied(30, 130, 70, 30)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(150, 40, 40, 30)
                        };
                        let available = ui.available_rect_before_wrap();
                        ui.painter().rect_filled(available, CornerRadius::ZERO, bg);
                        ui.label(
                            egui::RichText::new(format!("{sign}${:.0}", result.pnl))
                                .color(color)
                                .size(13.0)
                                .strong(),
                        );
                    });
                });
            }
        });

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(
            "Each scenario re-prices every position under shocked spot/vol/rate \
             and sums the P&L impact.",
        )
        .color(DIM)
        .size(10.0),
    );
}
