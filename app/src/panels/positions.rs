use crate::state::AppState;
use egui_extras::{Column, TableBuilder};
use risk_core::{greeks, OptionType};

pub fn positions_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Positions & Greeks");
    ui.add_space(4.0);

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(60.0)) // Symbol
        .column(Column::auto().at_least(50.0)) // Type
        .column(Column::auto().at_least(70.0)) // Spot
        .column(Column::auto().at_least(70.0)) // Strike
        .column(Column::auto().at_least(60.0)) // Expiry (y)
        .column(Column::auto().at_least(60.0)) // Qty
        .column(Column::auto().at_least(75.0)) // Price
        .column(Column::auto().at_least(70.0)) // Delta
        .column(Column::auto().at_least(70.0)) // Gamma
        .column(Column::auto().at_least(70.0)) // Vega
        .column(Column::auto().at_least(70.0)) // Theta
        .column(Column::auto().at_least(70.0)) // Rho
        .header(20.0, |mut header| {
            for label in [
                "Symbol", "Type", "Spot", "Strike", "Exp (y)", "Qty", "Price", "Delta", "Gamma",
                "Vega", "Theta", "Rho",
            ] {
                header.col(|ui| {
                    ui.strong(label);
                });
            }
        })
        .body(|mut body| {
            for pos in &state.positions {
                let inputs = pos.bs_inputs();
                let price = inputs.price();
                let g = greeks(&inputs);
                body.row(22.0, |mut row| {
                    row.col(|ui| {
                        ui.label(&pos.underlying_symbol);
                    });
                    row.col(|ui| {
                        ui.label(match pos.option_type {
                            OptionType::Call => "Call",
                            OptionType::Put => "Put",
                        });
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2}", pos.spot));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2}", pos.strike));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2}", pos.time_to_expiry));
                    });
                    row.col(|ui| {
                        let color = if pos.quantity >= 0.0 {
                            egui::Color32::from_rgb(90, 200, 120)
                        } else {
                            egui::Color32::from_rgb(220, 100, 100)
                        };
                        ui.colored_label(color, format!("{:+.0}", pos.quantity));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2}", price));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.3}", g.delta));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.4}", g.gamma));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.3}", g.vega / 100.0));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.3}", g.theta / 365.0));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.3}", g.rho / 100.0));
                    });
                });
            }
        });

    ui.add_space(8.0);
    ui.separator();
    ui.strong("Portfolio Aggregate Greeks");
    egui::Grid::new("agg_greeks_grid").num_columns(5).show(ui, |ui| {
        ui.label("Delta");
        ui.label("Gamma");
        ui.label("Vega (1%)");
        ui.label("Theta (1d)");
        ui.label("Rho (1%)");
        ui.end_row();
        ui.strong(format!("{:.1}", state.portfolio_greeks.delta));
        ui.strong(format!("{:.2}", state.portfolio_greeks.gamma));
        ui.strong(format!("{:.1}", state.portfolio_greeks.vega / 100.0));
        ui.strong(format!("{:.1}", state.portfolio_greeks.theta / 365.0));
        ui.strong(format!("{:.1}", state.portfolio_greeks.rho / 100.0));
        ui.end_row();
    });
}
