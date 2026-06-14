/// Módulo de estadísticas para validación
use std::f64;

/// Calcula la media de un vector
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Calcula la desviación estándar
pub fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|&v| (v - m).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

/// Calcula la mediana
pub fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    }
}

/// Calcula el percentil p (0-100)
pub fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rank = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let frac = rank - lower as f64;
        sorted[lower] * (1.0 - frac) + sorted[upper] * frac
    }
}

/// Test de Wilcoxon signed-rank (simplificado)
/// Devuelve (W+, W-, p-value aproximada)
pub fn wilcoxon_signed_rank(a: &[f64], b: &[f64]) -> (f64, f64, f64) {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    if n == 0 {
        return (0.0, 0.0, 1.0);
    }

    let mut diffs: Vec<(f64, bool)> = Vec::new();
    for i in 0..n {
        let d = a[i] - b[i];
        if d.abs() > 1e-10 {
            diffs.push((d.abs(), d > 0.0));
        }
    }

    diffs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut w_plus = 0.0;
    let mut w_minus = 0.0;
    let mut i = 0;
    while i < diffs.len() {
        let mut j = i;
        while j < diffs.len() && (diffs[j].0 - diffs[i].0).abs() < 1e-10 {
            j += 1;
        }
        let avg_rank = (i + 1 + j) as f64 / 2.0;
        for k in i..j {
            if diffs[k].1 {
                w_plus += avg_rank;
            } else {
                w_minus += avg_rank;
            }
        }
        i = j;
    }

    // p-value aproximada usando normal para n > 20
    let w = w_plus.min(w_minus);
    let n_eff = diffs.len() as f64;
    if n_eff < 10.0 {
        return (w_plus, w_minus, 1.0);
    }
    let mu = n_eff * (n_eff + 1.0) / 4.0;
    let sigma = (n_eff * (n_eff + 1.0) * (2.0 * n_eff + 1.0) / 24.0).sqrt();
    let z = (w - mu) / sigma;
    let p = 2.0 * (1.0 - normal_cdf(z.abs()));

    (w_plus, w_minus, p)
}

/// CDF de la distribución normal estándar (aproximación)
fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / 2.0_f64.sqrt()))
}

/// Función de error (aproximación de Abramowitz y Stegun)
fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Coeficiente de correlación de Pearson
pub fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len());
    let n = x.len() as f64;
    if n < 2.0 {
        return 0.0;
    }

    let mean_x = mean(x);
    let mean_y = mean(y);

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    let den = (den_x * den_y).sqrt();
    if den < 1e-10 {
        0.0
    } else {
        num / den
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        assert!((mean(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_median() {
        assert!((median(&[1.0, 3.0, 2.0, 5.0, 4.0]) - 3.0).abs() < 1e-10);
        assert!((median(&[1.0, 2.0, 3.0, 4.0]) - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_percentile() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile(&v, 50.0) - 3.0).abs() < 1e-10);
        assert!((percentile(&v, 0.0) - 1.0).abs() < 1e-10);
        assert!((percentile(&v, 100.0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_pearson() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert!((pearson_correlation(&x, &y) - 1.0).abs() < 1e-10);
    }
}
