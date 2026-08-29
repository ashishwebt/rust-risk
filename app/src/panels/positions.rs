use crate::state::AppState;
use egui::epaint::CornerRadius;
use egui_extras::{Column, TableBuilder};
use risk_core::{greeks, OptionType};

// Accent color for panel headers
const HEADER_ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 140, 255);
const HEADER_DIM: egui::Color32 = egui::Color32::from_rgb(100, 115, 140);
const POSITIVE: egui::Color32 = egui::Color32::from_rgb(60, 210, 120);
const NEGATIVE: egui::Color32 = egui::Color32::from_rgb(240, 90, 90);

pub fn positions_panel(ui: &mut egui::Ui, state: &AppState) {
    panel_header(ui, "📋  Positions & Greeks");
    ui.add_space(8.0);

    TableBuilder::new(ui)
        .id_salt("positions_table")
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(64.0)) // Symbol
        .column(Column::auto().at_least(48.0)) // Type
        .column(Column::auto().at_least(74.0)) // Spot
        .column(Column::auto().at_least(74.0)) // Strike
        .column(Column::auto().at_least(60.0)) // Expiry
        .column(Column::auto().at_least(60.0)) // Qty
        .column(Column::auto().at_least(74.0)) // Price
        .column(Column::auto().at_least(72.0)) // Delta
        .column(Column::auto().at_least(72.0)) // Gamma
        .column(Column::auto().at_least(72.0)) // Vega
        .column(Column::auto().at_least(72.0)) // Theta
        .column(Column::auto().at_least(72.0)) // Rho
        .header(24.0, |mut header| {
            for label in [
                "Symbol", "Type", "Spot", "Strike", "Exp (y)", "Qty",
                "Price", "Delta", "Gamma", "Vega", "Theta", "Rho",
            ] {
                header.col(|ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .color(HEADER_DIM)
                            .size(11.0)
                            .strong(),
                    );
                });
            }
        })
        .body(|mut body| {
            for pos in &state.positions {
                let inputs = pos.bs_inputs();
                let price = inputs.price();
                let g = greeks(&inputs);

                body.row(26.0, |mut row| {
                    row.col(|ui| {
                        ui.label(
                            egui::RichText::new(&pos.underlying_symbol)
                                .color(egui::Color32::from_rgb(200, 215, 240))
                                .strong(),
                        );
                    });
                    row.col(|ui| {
                        let (text, color) = match pos.option_type {
                            OptionType::Call => ("Call", egui::Color32::from_rgb(80, 190, 255)),
                            OptionType::Put => ("Put", egui::Color32::from_rgb(255, 140, 80)),
                        };
                        ui.label(egui::RichText::new(text).color(color).size(12.0));
                    });
                    row.col(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.2}", pos.spot))
                                .color(egui::Color32::from_rgb(200, 210, 230)),
                        );
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2}", pos.strike));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2}", pos.time_to_expiry));
                    });
                    row.col(|ui| {
                        let color = if pos.quantity >= 0.0 { POSITIVE } else { NEGATIVE };
                        ui.label(
                            egui::RichText::new(format!("{:+.0}", pos.quantity))
                                .color(color)
                                .strong(),
                        );
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2}", price));
                    });
                    row.col(|ui| {
                        ui.label(greek_text(g.delta, 3));
                    });
                    row.col(|ui| {
                        ui.label(egui::RichText::new(format!("{:.4}", g.gamma))
                            .color(egui::Color32::from_rgb(160, 175, 200)));
                    });
                    row.col(|ui| {
                        ui.label(egui::RichText::new(format!("{:.3}", g.vega / 100.0))
                            .color(egui::Color32::from_rgb(160, 175, 200)));
                    });
                    row.col(|ui| {
                        let theta = g.theta / 365.0;
                        let color = if theta >= 0.0 { POSITIVE } else { NEGATIVE };
                        ui.label(
                            egui::RichText::new(format!("{:.3}", theta)).color(color),
                        );
                    });
                    row.col(|ui| {
                        ui.label(egui::RichText::new(format!("{:.3}", g.rho / 100.0))
                            .color(egui::Color32::from_rgb(160, 175, 200)));
                    });
                });
            }
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(6.0);

    ui.label(
        egui::RichText::new("Portfolio Aggregate Greeks")
            .color(HEADER_ACCENT)
            .size(12.0)
            .strong(),
    );
    ui.add_space(4.0);

    egui::Grid::new("agg_greeks_grid")
        .num_columns(5)
        .spacing([20.0, 6.0])
        .show(ui, |ui| {
            for label in ["Delta", "Gamma", "Vega (1%)", "Theta (1d)", "Rho (1%)"] {
                ui.label(
                    egui::RichText::new(label)
                        .color(HEADER_DIM)
                        .size(11.0),
                );
            }
            ui.end_row();

            let pg = &state.portfolio_greeks;
            for (val, decimals) in [
                (pg.delta, 1usize),
                (pg.gamma, 2),
                (pg.vega / 100.0, 1),
                (pg.theta / 365.0, 1),
                (pg.rho / 100.0, 1),
            ] {
                ui.label(
                    egui::RichText::new(format!("{:.prec$}", val, prec = decimals))
                        .color(egui::Color32::from_rgb(220, 230, 255))
                        .size(14.0)
                        .strong(),
                );
            }
            ui.end_row();
        });
}

/// Panel section header with a colored accent stripe on the left.
pub fn panel_header(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 20.0), egui::Sense::hover());
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(2),
            HEADER_ACCENT,
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(title)
                .size(15.0)
                .color(egui::Color32::from_rgb(220, 230, 255))
                .strong(),
        );
    });
}

/// Color-code a greek value: positive = blue-white, negative = orange-ish.
fn greek_text(val: f64, decimals: usize) -> egui::RichText {
    let color = if val > 0.05 {
        egui::Color32::from_rgb(100, 190, 255)
    } else if val < -0.05 {
        egui::Color32::from_rgb(255, 150, 80)
    } else {
        egui::Color32::from_rgb(180, 185, 200)
    };
    egui::RichText::new(format!("{:.prec$}", val, prec = decimals)).color(color)
}
