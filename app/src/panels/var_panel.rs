use crate::state::AppState;
use egui::epaint::CornerRadius;
use super::positions::panel_header;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 140, 255);
const DIM: egui::Color32 = egui::Color32::from_rgb(110, 125, 150);

pub fn var_panel(ui: &mut egui::Ui, state: &mut AppState) {
    panel_header(ui, "📊  Value at Risk");
    ui.add_space(8.0);

    // Config row
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Confidence:").color(DIM).size(12.0));
        for (label, level) in [("90%", 0.90f64), ("95%", 0.95), ("99%", 0.99)] {
            let selected = (state.var_config.confidence - level).abs() < 1e-9;
            let text = if selected {
                egui::RichText::new(label)
                    .color(egui::Color32::from_rgb(100, 200, 255))
                    .strong()
            } else {
                egui::RichText::new(label).color(egui::Color32::from_rgb(160, 175, 195))
            };
            if ui.selectable_label(selected, text).clicked() {
                state.var_config.confidence = level;
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label(egui::RichText::new("Horizon:").color(DIM).size(12.0));
        ui.add(
            egui::DragValue::new(&mut state.var_config.horizon_days)
                .range(1.0..=30.0)
                .speed(0.1)
                .suffix(" days"),
        );
    });

    ui.add_space(12.0);

    // Compute portfolio value once for % display
    let portfolio_value: f64 = state
        .active_positions()
        .iter()
        .map(|p| p.bs_inputs().price() * p.notional_quantity())
        .sum::<f64>()
        .abs();

    // Big VaR numbers
    ui.horizontal(|ui| {
        var_card(ui, "Historical Sim", state.portfolio_historical_var(), portfolio_value);
        ui.add_space(16.0);
        var_card(ui, "Parametric (Var-Cov)", state.portfolio_parametric_var(), portfolio_value);
    });

    ui.add_space(10.0);

    let n_obs = state
        .pnl_history
        .values()
        .map(|v| v.len())
        .max()
        .unwrap_or(0);

    ui.label(
        egui::RichText::new(format!(
            "Historical VaR based on {n_obs} simulated P&L observations."
        ))
        .color(DIM)
        .size(11.0),
    );
}

/// A self-contained card showing a VaR figure prominently.
fn var_card(ui: &mut egui::Ui, label: &str, value: f64, portfolio_value: f64) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(12, 16, 26))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(38, 52, 80)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(14i8)
        .show(ui, |ui| {
            ui.set_min_width(160.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(label).color(DIM).size(11.0));
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("${:.0}", value))
                        .color(egui::Color32::from_rgb(255, 180, 60))
                        .size(26.0)
                        .strong(),
                );
                ui.add_space(2.0);
                let pct_text = if portfolio_value > 0.0 {
                    format!("({:.1}% of portfolio)", value / portfolio_value * 100.0)
                } else {
                    "—".to_string()
                };
                ui.label(
                    egui::RichText::new(pct_text)
                        .color(ACCENT)
                        .size(10.0),
                );
            });
        });
}
