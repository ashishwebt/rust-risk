use crate::state::AppState;
use egui_extras::{Column, TableBuilder};

pub fn vol_surface_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Implied Volatility Surface");
    ui.add_space(4.0);

    let surf = &state.vol_surface;
    let mut builder = TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(70.0));
    for _ in &surf.strikes {
        builder = builder.column(Column::auto().at_least(65.0));
    }

    builder
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Exp \\ Strike");
            });
            for k in &surf.strikes {
                header.col(|ui| {
                    ui.strong(format!("{k:.0}"));
                });
            }
        })
        .body(|mut body| {
            for (ei, expiry) in surf.expiries.iter().enumerate() {
                body.row(22.0, |mut row| {
                    row.col(|ui| {
                        ui.label(format!("{:.2}y", expiry));
                    });
                    for ki in 0..surf.strikes.len() {
                        let vol = surf.vols.get(ei).and_then(|r| r.get(ki)).copied().unwrap_or(0.0);
                        row.col(|ui| {
                            let color = heat_color(vol);
                            ui.colored_label(color, format!("{:.1}%", vol * 100.0));
                        });
                    }
                });
            }
        });

    ui.add_space(6.0);
    ui.small("Bilinear-interpolated between grid points; used to price/mark positions between quoted strikes and tenors.");
}

/// Simple green (low vol) -> red (high vol) heat color for the surface.
fn heat_color(vol: f64) -> egui::Color32 {
    let t = ((vol - 0.15) / (0.45 - 0.15)).clamp(0.0, 1.0) as f32;
    let r = (100.0 + t * 130.0) as u8;
    let g = (200.0 - t * 130.0) as u8;
    egui::Color32::from_rgb(r, g, 90)
}
