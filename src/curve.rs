//! Interpolación Hermite cúbica monótona (PCHIP, Fritsch–Carlson).
//! C¹, sin sobre-oscilación; pendiente 0 en extremos locales (pico de descarga).
#[derive(Clone, Copy)]
pub struct Seg { pub t0: f64, pub t1: f64, pub a: f64, pub b: f64, pub c: f64, pub d: f64 }
pub struct Pchip { pub segs: Vec<Seg> }

fn edge(h0: f64, h1: f64, s0: f64, s1: f64) -> f64 {
    let mut e = ((2.0 * h0 + h1) * s0 - h0 * s1) / (h0 + h1);
    if e * s0 <= 0.0 { e = 0.0 } else if s0 * s1 <= 0.0 && e.abs() > 3.0 * s0.abs() { e = 3.0 * s0 }
    e
}

impl Pchip {
    /// Requiere ≥ 3 puntos con t estrictamente creciente.
    pub fn new(p: &[(f64, f64)]) -> Self {
        let n = p.len();
        let h: Vec<f64> = (0..n - 1).map(|i| p[i + 1].0 - p[i].0).collect();
        let s: Vec<f64> = (0..n - 1).map(|i| (p[i + 1].1 - p[i].1) / h[i]).collect();
        let mut m = vec![0.0; n];
        for i in 1..n - 1 {
            if s[i - 1] * s[i] > 0.0 {
                let (a, b) = (2.0 * h[i] + h[i - 1], h[i] + 2.0 * h[i - 1]);
                m[i] = (a + b) / (a / s[i - 1] + b / s[i]);
            }
        }
        m[0] = edge(h[0], h[1], s[0], s[1]);
        m[n - 1] = edge(h[n - 2], h[n - 3], s[n - 2], s[n - 3]);
        let segs = (0..n - 1)
            .map(|i| Seg {
                t0: p[i].0, t1: p[i + 1].0, a: p[i].1, b: m[i],
                c: (3.0 * s[i] - 2.0 * m[i] - m[i + 1]) / h[i],
                d: (m[i] + m[i + 1] - 2.0 * s[i]) / (h[i] * h[i]),
            })
            .collect();
        Pchip { segs }
    }
    pub fn at(&self, t: f64) -> f64 {
        let s = self.segs.iter().find(|s| t <= s.t1).unwrap_or(self.segs.last().unwrap());
        let u = t - s.t0;
        s.a + u * (s.b + u * (s.c + u * s.d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pasa_por_los_puntos_y_es_monotona_al_calentar() {
        let p = [(0., 30.), (4., 90.), (9., 158.), (13., 203.), (16., 40.)];
        let f = Pchip::new(&p);
        for q in p { assert!((f.at(q.0) - q.1).abs() < 1e-9); }
        let mut prev = f.at(0.0);
        for i in 1..=1300 { let v = f.at(i as f64 / 100.0); assert!(v >= prev - 1e-9); prev = v; }
        assert!(f.at(14.5) < f.at(13.0));
    }
}
