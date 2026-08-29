use crate::state::AppState;
use egui_extras::{Column, TableBuilder};

pub fn stress_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Stress Scenarios");
    ui.add_space(4.0);

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(220.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(100.0))
        .header(20.0, |mut header| {
            for label in ["Scenario", "Base Value", "Stressed Value", "P&L"] {
                header.col(|ui| {
                    ui.strong(label);
                });
            }
        })
        .body(|mut body| {
            for result in &state.scenario_results {
                body.row(22.0, |mut row| {
                    row.col(|ui| {
                        ui.label(&result.scenario_name);
                    });
                    row.col(|ui| {
                        ui.label(format!("${:.0}", result.base_value));
                    });
                    row.col(|ui| {
                        ui.label(format!("${:.0}", result.stressed_value));
                    });
                    row.col(|ui| {
                        let color = if result.pnl >= 0.0 {
                            egui::Color32::from_rgb(90, 200, 120)
                        } else {
                            egui::Color32::from_rgb(220, 100, 100)
                        };
                        ui.colored_label(color, format!("{:+.0}", result.pnl));
                    });
                });
            }
        });

    ui.add_space(6.0);
    ui.small("Each scenario re-prices every position under shocked spot/vol/rate and sums the P&L impact.");
}
