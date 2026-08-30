use crate::state::{AppState, FormOptionType};
use egui::epaint::CornerRadius;
use risk_core::Position;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 140, 255);
const ERROR_COLOR: egui::Color32 = egui::Color32::from_rgb(240, 90, 90);
const DIM: egui::Color32 = egui::Color32::from_rgb(120, 135, 160);

/// Draw the "Add Position" toggle button + inline form panel.
///
/// Call this just below the positions table inside the left column.
pub fn symbol_form_panel(ui: &mut egui::Ui, state: &mut AppState) {
    // ── Toggle button ────────────────────────────────────────────────────
    ui.add_space(8.0);
    let btn_label = if state.position_form.open {
        "✕  Cancel"
    } else {
        "＋  Add Position"
    };
    let btn = egui::Button::new(
        egui::RichText::new(btn_label)
            .size(12.0)
            .color(if state.position_form.open {
                egui::Color32::from_rgb(240, 120, 80)
            } else {
                egui::Color32::from_rgb(100, 200, 120)
            }),
    )
    .corner_radius(CornerRadius::same(4));

    if ui.add(btn).clicked() {
        state.position_form.open = !state.position_form.open;
        // Reset error on re-open
        state.position_form.error = None;
    }

    if !state.position_form.open {
        return;
    }

    ui.add_space(6.0);

    // ── Form frame ───────────────────────────────────────────────────────
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(22, 28, 42))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 65, 95)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(12i8)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("New Position")
                    .color(ACCENT)
                    .size(13.0)
                    .strong(),
            );
            ui.add_space(8.0);

            egui::Grid::new("add_pos_grid")
                .num_columns(4)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    // Row 1: Symbol | Option type
                    field_label(ui, "Symbol");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.position_form.symbol)
                            .desired_width(90.0)
                            .hint_text("e.g. TSLA"),
                    );

                    field_label(ui, "Type");
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut state.position_form.option_type,
                            FormOptionType::Call,
                            egui::RichText::new("Call").color(egui::Color32::from_rgb(80, 190, 255)),
                        );
                        ui.selectable_value(
                            &mut state.position_form.option_type,
                            FormOptionType::Put,
                            egui::RichText::new("Put").color(egui::Color32::from_rgb(255, 140, 80)),
                        );
                    });
                    ui.end_row();

                    // Row 2: Strike | Expiry (years)
                    field_label(ui, "Strike");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.position_form.strike)
                            .desired_width(90.0),
                    );

                    field_label(ui, "Expiry (y)");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.position_form.expiry)
                            .desired_width(90.0)
                            .hint_text("0.25"),
                    );
                    ui.end_row();

                    // Row 3: Quantity | Contract multiplier
                    field_label(ui, "Quantity");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.position_form.quantity)
                            .desired_width(90.0)
                            .hint_text("e.g. 10 or -5"),
                    );

                    field_label(ui, "Multiplier");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.position_form.contract_multiplier)
                            .desired_width(90.0)
                            .hint_text("100"),
                    );
                    ui.end_row();

                    // Row 4: Volatility | Rate
                    field_label(ui, "Volatility");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.position_form.volatility)
                            .desired_width(90.0)
                            .hint_text("0.25"),
                    );

                    field_label(ui, "Rate");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.position_form.rate)
                            .desired_width(90.0)
                            .hint_text("0.045"),
                    );
                    ui.end_row();

                    // Row 5: Dividend yield (spans half row)
                    field_label(ui, "Div Yield");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.position_form.dividend_yield)
                            .desired_width(90.0)
                            .hint_text("0.0"),
                    );
                    ui.label(""); // filler
                    ui.label(""); // filler
                    ui.end_row();
                });

            // ── Validation error ─────────────────────────────────────────
            if let Some(err) = &state.position_form.error.clone() {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("⚠  {err}")).color(ERROR_COLOR).size(11.0));
            }

            ui.add_space(8.0);

            // ── Submit button ────────────────────────────────────────────
            let submit = egui::Button::new(
                egui::RichText::new("Add to Portfolio")
                    .color(egui::Color32::from_rgb(120, 220, 140))
                    .size(12.0),
            )
            .corner_radius(CornerRadius::same(4))
            .fill(egui::Color32::from_rgb(28, 55, 38));

            if ui.add(submit).clicked() {
                match build_position(&state.position_form) {
                    Ok(pos) => {
                        state.position_form.error = None;
                        state.position_form.open = false;
                        // Reset form for next use
                        let ot = state.position_form.option_type;
                        state.position_form = crate::state::PositionForm::default();
                        state.position_form.option_type = ot;
                        state.add_position(pos);
                    }
                    Err(e) => {
                        state.position_form.error = Some(e);
                    }
                }
            }
        });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(DIM).size(11.0));
}

/// Parse form strings into a `Position`, returning a user-facing error string on failure.
fn build_position(form: &crate::state::PositionForm) -> Result<Position, String> {
    let symbol = form.symbol.trim().to_uppercase();
    if symbol.is_empty() {
        return Err("Symbol is required".into());
    }

    let strike = form.strike.trim().parse::<f64>().map_err(|_| "Invalid strike")?;
    let expiry = form.expiry.trim().parse::<f64>().map_err(|_| "Invalid expiry")?;
    let volatility = form.volatility.trim().parse::<f64>().map_err(|_| "Invalid volatility")?;
    let rate = form.rate.trim().parse::<f64>().map_err(|_| "Invalid rate")?;
    let div = form.dividend_yield.trim().parse::<f64>().map_err(|_| "Invalid dividend yield")?;
    let quantity = form.quantity.trim().parse::<f64>().map_err(|_| "Invalid quantity")?;
    let multiplier = form
        .contract_multiplier
        .trim()
        .parse::<f64>()
        .map_err(|_| "Invalid contract multiplier")?;

    if strike <= 0.0 { return Err("Strike must be > 0".into()); }
    if expiry <= 0.0 { return Err("Expiry must be > 0".into()); }
    if volatility <= 0.0 { return Err("Volatility must be > 0".into()); }
    if multiplier <= 0.0 { return Err("Multiplier must be > 0".into()); }
    if quantity == 0.0 { return Err("Quantity cannot be zero".into()); }

    Ok(Position {
        id: 0, // DB assigns the real id
        underlying_symbol: symbol,
        spot: 0.0, // will be replaced by first live tick
        strike,
        time_to_expiry: expiry,
        rate,
        dividend_yield: div,
        volatility,
        option_type: form.option_type.into(),
        quantity,
        contract_multiplier: multiplier,
    })
}
