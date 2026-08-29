/// Confidence level for VaR calculations, e.g. 0.95 or 0.99.
#[derive(Debug, Clone, Copy)]
pub struct VarConfig {
    pub confidence: f64,
    /// Holding period in days used to scale 1-day figures (sqrt-of-time rule).
    pub horizon_days: f64,
}

impl Default for VarConfig {
    fn default() -> Self {
        Self {
            confidence: 0.95,
            horizon_days: 1.0,
        }
    }
}

/// Historical simulation VaR: given a series of historical P&L (or return * exposure)
/// observations, returns the loss (positive number) at the given confidence level.
pub fn historical_var(pnl_history: &[f64], config: &VarConfig) -> f64 {
    if pnl_history.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = pnl_history.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((1.0 - config.confidence) * sorted.len() as f64).floor() as usize;
    let idx = idx.min(sorted.len() - 1);
    let quantile_pnl = sorted[idx];
    let one_day_var = -quantile_pnl.min(0.0).min(quantile_pnl);
    (one_day_var.max(0.0)) * config.horizon_days.sqrt()
}

/// Expected Shortfall (CVaR): average loss beyond the VaR threshold.
pub fn expected_shortfall(pnl_history: &[f64], config: &VarConfig) -> f64 {
    if pnl_history.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = pnl_history.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let cutoff = ((1.0 - config.confidence) * sorted.len() as f64).ceil() as usize;
    let cutoff = cutoff.max(1).min(sorted.len());
    let tail = &sorted[0..cutoff];
    let avg = tail.iter().sum::<f64>() / tail.len() as f64;
    (-avg.min(0.0)) * config.horizon_days.sqrt()
}

/// Parametric (variance-covariance) VaR assuming normally distributed returns.
/// `portfolio_value` and `daily_volatility` (as a fraction, e.g. 0.015 for 1.5%/day)
/// characterize the position; z-scores for common confidence levels are used.
pub fn parametric_var(portfolio_value: f64, daily_volatility: f64, config: &VarConfig) -> f64 {
    let z = z_score(config.confidence);
    portfolio_value * daily_volatility * z * config.horizon_days.sqrt()
}

fn z_score(confidence: f64) -> f64 {
    // A few common lookups; falls back to a rational approximation otherwise.
    if (confidence - 0.90).abs() < 1e-6 {
        1.2816
    } else if (confidence - 0.95).abs() < 1e-6 {
        1.6449
    } else if (confidence - 0.975).abs() < 1e-6 {
        1.9600
    } else if (confidence - 0.99).abs() < 1e-6 {
        2.3263
    } else {
        inverse_norm_cdf(confidence)
    }
}

/// Acklam's algorithm approximation of the inverse standard normal CDF.
fn inverse_norm_cdf(p: f64) -> f64 {
    let p = p.clamp(1e-10, 1.0 - 1e-10);
    let a = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    let b = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    let c = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    let d = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];
    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_var_picks_tail_loss() {
        let pnl = vec![-100.0, -50.0, -10.0, 5.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0];
        let cfg = VarConfig {
            confidence: 0.90,
            horizon_days: 1.0,
        };
        let v = historical_var(&pnl, &cfg);
        assert!(v > 0.0);
    }

    #[test]
    fn parametric_var_scales_with_horizon() {
        let cfg1 = VarConfig {
            confidence: 0.95,
            horizon_days: 1.0,
        };
        let cfg10 = VarConfig {
            confidence: 0.95,
            horizon_days: 10.0,
        };
        let v1 = parametric_var(1_000_000.0, 0.015, &cfg1);
        let v10 = parametric_var(1_000_000.0, 0.015, &cfg10);
        assert!((v10 - v1 * 10f64.sqrt()).abs() < 1e-6);
    }
}
