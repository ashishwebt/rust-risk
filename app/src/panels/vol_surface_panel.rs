use crate::state::AppState;
use egui::epaint::CornerRadius;
use egui_extras::{Column, TableBuilder};
use super::positions::panel_header;

const DIM: egui::Color32 = egui::Color32::from_rgb(100, 115, 140);

pub fn vol_surface_panel(ui: &mut egui::Ui, state: &AppState) {
    panel_header(ui, "🌡  Implied Volatility Surface");
    ui.add_space(8.0);

    let surf = &state.vol_surface;

    let mut builder = TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(72.0));

    for _ in &surf.strikes {
        builder = builder.column(Column::auto().at_least(68.0));
    }

    builder
        .header(24.0, |mut header| {
            header.col(|ui| {
                ui.label(
                    egui::RichText::new("Exp \\ Strike")
                        .color(DIM)
                        .size(11.0)
                        .strong(),
                );
            });
            for k in &surf.strikes {
                header.col(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{k:.0}"))
                            .color(DIM)
                            .size(11.0)
                            .strong(),
                    );
                });
            }
        })
        .body(|mut body| {
            for (ei, expiry) in surf.expiries.iter().enumerate() {
                body.row(26.0, |mut row| {
                    row.col(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.2}y", expiry))
                                .color(egui::Color32::from_rgb(180, 190, 215)),
                        );
                    });
                    for ki in 0..surf.strikes.len() {
                        let vol = surf
                            .vols
                            .get(ei)
                            .and_then(|r| r.get(ki))
                            .copied()
                            .unwrap_or(0.0);
                        row.col(|ui| {
                            let (bg, fg) = heat_colors(vol);
                            let available = ui.available_rect_before_wrap();
                            ui.painter().rect_filled(available, CornerRadius::ZERO, bg);
                            ui.label(
                                egui::RichText::new(format!("{:.1}%", vol * 100.0))
                                    .color(fg)
                                    .size(12.0)
                                    .strong(),
                            );
                        });
                    }
                });
            }
        });

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(
            "Bilinear-interpolated between grid points; \
             used to price/mark positions between quoted strikes and tenors.",
        )
        .color(DIM)
        .size(10.0),
    );

    // Color legend
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Low vol ").color(DIM).size(10.0));
        for pct in [15u32, 20, 25, 30, 35, 40, 45] {
            let v = pct as f64 / 100.0;
            let (bg, _) = heat_colors(v);
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(20.0, 12.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, CornerRadius::same(2), bg);
        }
        ui.label(egui::RichText::new(" High vol").color(DIM).size(10.0));
    });
}

/// Returns (background fill, foreground text) colors for a given vol level.
fn heat_colors(vol: f64) -> (egui::Color32, egui::Color32) {
    let t = ((vol - 0.12) / (0.50 - 0.12)).clamp(0.0, 1.0) as f32;

    // Background: deep blue (low vol) → amber (mid) → red (high)
    let bg = if t < 0.5 {
        let s = t * 2.0;
        egui::Color32::from_rgba_unmultiplied(
            (20.0 + s * 80.0) as u8,
            (60.0 + s * 40.0) as u8,
            (120.0 - s * 60.0) as u8,
            60,
        )
    } else {
        let s = (t - 0.5) * 2.0;
        egui::Color32::from_rgba_unmultiplied(
            (100.0 + s * 130.0) as u8,
            (100.0 - s * 60.0) as u8,
            (60.0 - s * 40.0) as u8,
            70,
        )
    };

    // Foreground text: bright for readability
    let fg = if t < 0.4 {
        egui::Color32::from_rgb(120, 210, 255)
    } else if t < 0.7 {
        egui::Color32::from_rgb(255, 210, 80)
    } else {
        egui::Color32::from_rgb(255, 110, 90)
    };

    (bg, fg)
}
