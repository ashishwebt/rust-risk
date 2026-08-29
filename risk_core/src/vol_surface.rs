/// A simple (strike, expiry) implied-vol grid with bilinear interpolation.
/// Strikes and expiries (in years) must each be stored sorted ascending.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VolSurface {
    pub strikes: Vec<f64>,
    pub expiries: Vec<f64>,
    /// Row-major: vols[expiry_idx][strike_idx]
    pub vols: Vec<Vec<f64>>,
}

impl VolSurface {
    pub fn new(strikes: Vec<f64>, expiries: Vec<f64>) -> Self {
        let vols = vec![vec![0.0; strikes.len()]; expiries.len()];
        Self {
            strikes,
            expiries,
            vols,
        }
    }

    pub fn set(&mut self, expiry_idx: usize, strike_idx: usize, vol: f64) {
        if let Some(row) = self.vols.get_mut(expiry_idx) {
            if let Some(cell) = row.get_mut(strike_idx) {
                *cell = vol;
            }
        }
    }

    /// Bilinear interpolation with clamping at the edges of the grid.
    pub fn interpolate(&self, strike: f64, expiry: f64) -> Option<f64> {
        if self.strikes.is_empty() || self.expiries.is_empty() {
            return None;
        }
        let (kx0, kx1, kt) = bracket(&self.strikes, strike);
        let (ex0, ex1, et) = bracket(&self.expiries, expiry);

        let v00 = self.vols.get(ex0)?.get(kx0)?;
        let v01 = self.vols.get(ex0)?.get(kx1)?;
        let v10 = self.vols.get(ex1)?.get(kx0)?;
        let v11 = self.vols.get(ex1)?.get(kx1)?;

        let top = v00 * (1.0 - kt) + v01 * kt;
        let bottom = v10 * (1.0 - kt) + v11 * kt;
        Some(top * (1.0 - et) + bottom * et)
    }
}

/// Finds the bracketing indices for `value` in a sorted ascending slice,
/// returning (lower_idx, upper_idx, fraction between them, clamped to [0,1]).
fn bracket(sorted: &[f64], value: f64) -> (usize, usize, f64) {
    if sorted.len() == 1 {
        return (0, 0, 0.0);
    }
    if value <= sorted[0] {
        return (0, 1, 0.0);
    }
    if value >= *sorted.last().unwrap() {
        let n = sorted.len();
        return (n - 2, n - 1, 1.0);
    }
    for i in 0..sorted.len() - 1 {
        if value >= sorted[i] && value <= sorted[i + 1] {
            let span = sorted[i + 1] - sorted[i];
            let t = if span.abs() < 1e-12 {
                0.0
            } else {
                (value - sorted[i]) / span
            };
            return (i, i + 1, t);
        }
    }
    (0, 1, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_between_grid_points() {
        let mut surf = VolSurface::new(vec![90.0, 100.0, 110.0], vec![0.25, 1.0]);
        surf.set(0, 0, 0.22);
        surf.set(0, 1, 0.20);
        surf.set(0, 2, 0.23);
        surf.set(1, 0, 0.25);
        surf.set(1, 1, 0.21);
        surf.set(1, 2, 0.26);

        let v = surf.interpolate(95.0, 0.25).unwrap();
        assert!((v - 0.21).abs() < 1e-9);
    }
}
