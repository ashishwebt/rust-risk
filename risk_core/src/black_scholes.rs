use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OptionType {
    Call,
    Put,
}

/// Standard normal PDF.
pub fn norm_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
}

/// Standard normal CDF via Abramowitz-Stegun style erf approximation.
pub fn norm_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    // Abramowitz and Stegun formula 7.1.26, max error ~1.5e-7
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * y
}

#[derive(Debug, Clone, Copy)]
pub struct BsInputs {
    pub spot: f64,
    pub strike: f64,
    /// Time to expiry in years.
    pub time_to_expiry: f64,
    /// Continuously compounded risk-free rate.
    pub rate: f64,
    /// Continuously compounded dividend / carry yield.
    pub dividend_yield: f64,
    /// Annualized volatility (e.g. 0.20 for 20%).
    pub volatility: f64,
    pub option_type: OptionType,
}

impl BsInputs {
    fn d1_d2(&self) -> (f64, f64) {
        let vt = self.volatility * self.time_to_expiry.sqrt();
        let d1 = ((self.spot / self.strike).ln()
            + (self.rate - self.dividend_yield + 0.5 * self.volatility * self.volatility)
                * self.time_to_expiry)
            / vt;
        let d2 = d1 - vt;
        (d1, d2)
    }

    pub fn price(&self) -> f64 {
        if self.time_to_expiry <= 0.0 {
            return match self.option_type {
                OptionType::Call => (self.spot - self.strike).max(0.0),
                OptionType::Put => (self.strike - self.spot).max(0.0),
            };
        }
        let (d1, d2) = self.d1_d2();
        let df_r = (-self.rate * self.time_to_expiry).exp();
        let df_q = (-self.dividend_yield * self.time_to_expiry).exp();
        match self.option_type {
            OptionType::Call => {
                self.spot * df_q * norm_cdf(d1) - self.strike * df_r * norm_cdf(d2)
            }
            OptionType::Put => {
                self.strike * df_r * norm_cdf(-d2) - self.spot * df_q * norm_cdf(-d1)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    /// Vega per 1 vol point (1.00 = 100%); divide by 100 for a 1% move.
    pub vega: f64,
    /// Theta per year; divide by 365 for per-day decay.
    pub theta: f64,
    /// Rho per 1.0 (100%) rate move; divide by 100 for a 1% move.
    pub rho: f64,
}

pub fn greeks(inputs: &BsInputs) -> Greeks {
    if inputs.time_to_expiry <= 0.0 {
        return Greeks::default();
    }
    let (d1, d2) = inputs.d1_d2();
    let df_r = (-inputs.rate * inputs.time_to_expiry).exp();
    let df_q = (-inputs.dividend_yield * inputs.time_to_expiry).exp();
    let sqrt_t = inputs.time_to_expiry.sqrt();
    let pdf_d1 = norm_pdf(d1);

    let gamma = df_q * pdf_d1 / (inputs.spot * inputs.volatility * sqrt_t);
    let vega = inputs.spot * df_q * pdf_d1 * sqrt_t;

    let (delta, theta, rho) = match inputs.option_type {
        OptionType::Call => {
            let delta = df_q * norm_cdf(d1);
            let theta = -inputs.spot * df_q * pdf_d1 * inputs.volatility / (2.0 * sqrt_t)
                - inputs.rate * inputs.strike * df_r * norm_cdf(d2)
                + inputs.dividend_yield * inputs.spot * df_q * norm_cdf(d1);
            let rho = inputs.strike * inputs.time_to_expiry * df_r * norm_cdf(d2);
            (delta, theta, rho)
        }
        OptionType::Put => {
            let delta = df_q * (norm_cdf(d1) - 1.0);
            let theta = -inputs.spot * df_q * pdf_d1 * inputs.volatility / (2.0 * sqrt_t)
                + inputs.rate * inputs.strike * df_r * norm_cdf(-d2)
                - inputs.dividend_yield * inputs.spot * df_q * norm_cdf(-d1);
            let rho = -inputs.strike * inputs.time_to_expiry * df_r * norm_cdf(-d2);
            (delta, theta, rho)
        }
    };

    Greeks {
        delta,
        gamma,
        vega,
        theta,
        rho,
    }
}

/// Back out implied volatility from a market price via Newton-Raphson,
/// falling back to bisection if the derivative-based step misbehaves.
pub fn implied_vol(target_price: f64, mut inputs: BsInputs) -> Option<f64> {
    if inputs.time_to_expiry <= 0.0 || target_price <= 0.0 {
        return None;
    }
    let mut sigma = 0.3_f64;
    let mut lo = 0.0001_f64;
    let mut hi = 5.0_f64;

    for _ in 0..100 {
        inputs.volatility = sigma;
        let price = inputs.price();
        let g = greeks(&inputs);
        let diff = price - target_price;

        if diff.abs() < 1e-6 {
            return Some(sigma);
        }

        if diff > 0.0 {
            hi = sigma;
        } else {
            lo = sigma;
        }

        // Newton step; vega is per 1.00 vol unit here.
        if g.vega.abs() > 1e-8 {
            let next = sigma - diff / g.vega;
            if next > lo && next < hi {
                sigma = next;
                continue;
            }
        }
        // Bisection fallback.
        sigma = 0.5 * (lo + hi);
    }
    Some(sigma)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn atm_call_price_reasonable() {
        let inputs = BsInputs {
            spot: 100.0,
            strike: 100.0,
            time_to_expiry: 1.0,
            rate: 0.05,
            dividend_yield: 0.0,
            volatility: 0.2,
            option_type: OptionType::Call,
        };
        let price = inputs.price();
        assert_abs_diff_eq!(price, 10.4506, epsilon = 0.01);
    }

    #[test]
    fn implied_vol_round_trip() {
        let mut inputs = BsInputs {
            spot: 100.0,
            strike: 105.0,
            time_to_expiry: 0.5,
            rate: 0.03,
            dividend_yield: 0.01,
            volatility: 0.25,
            option_type: OptionType::Put,
        };
        let price = inputs.price();
        inputs.volatility = 0.0;
        let iv = implied_vol(price, inputs).unwrap();
        assert_abs_diff_eq!(iv, 0.25, epsilon = 1e-3);
    }
}
