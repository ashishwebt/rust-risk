use crate::state::AppState;

pub fn var_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Value at Risk");
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("Confidence:");
        for (label, level) in [("90%", 0.90), ("95%", 0.95), ("99%", 0.99)] {
            if ui
                .selectable_label(
                    (state.var_config.confidence - level).abs() < 1e-9,
                    label,
                )
                .clicked()
            {
                state.var_config.confidence = level;
            }
        }
        ui.separator();
        ui.label("Horizon (days):");
        ui.add(
            egui::DragValue::new(&mut state.var_config.horizon_days)
                .range(1.0..=30.0)
                .speed(0.1),
        );
    });

    ui.add_space(8.0);
    egui::Grid::new("var_grid").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
        ui.label("Historical Simulation VaR");
        ui.strong(format!("${:.0}", state.portfolio_historical_var()));
        ui.end_row();

        ui.label("Parametric (Var-Cov) VaR");
        ui.strong(format!("${:.0}", state.portfolio_parametric_var()));
        ui.end_row();
    });

    let n_obs = state.pnl_history.values().map(|v| v.len()).max().unwrap_or(0);
    ui.add_space(6.0);
    ui.small(format!(
        "Historical VaR based on {n_obs} simulated P&L observations (grows as the feed streams)."
    ));
}