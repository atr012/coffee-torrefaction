//! Modelo de datos: recetas, fases y estado de válvulas.
use crate::curve::Pchip;

pub const T_AXIS: f64 = 28.0; // eje de minutos fijo: 25 min máx. de tueste + 3 de enfriamiento
pub const ROAST_MIN: f64 = 13.0; // el tostador cilíndrico no tuesta en menos de 13 min
pub const ROAST_MAX: f64 = 25.0; // …ni más de 25 min (tueste oscuro)
pub const BEST: (f64, f64) = (15.0, 17.0); // mejores resultados
pub const STEP: f64 = 0.5; // paso mínimo de tiempo: 30 s
pub const NV: usize = 5; // nº de válvulas/actuadores (debe coincidir con VALVES)
pub const VALVES: [&str; NV] = ["SOL-BAJA", "SOL-ALTA", "BY-PASS", "CICLON", "ENFRIADOR"];

#[derive(Clone, Copy)]
pub enum End { At(f64), Discharge, Cooled }
#[derive(Clone)]
pub struct Phase { pub name: &'static str, pub end: End, pub v: [bool; NV] }
/// Intervalo de control de válvulas (rejilla fija: 1 min + cortes de fase fraccionarios).
#[derive(Clone)]
pub struct Slot { pub t0: f64, pub t1: f64, pub v: [bool; NV] }
#[derive(Clone)]
pub struct Recipe {
    pub title: &'static str, pub sub: &'static str, pub color: &'static str,
    pub tags: &'static str, pub tmax: f64,
    pub pts: [(f64, f64); 5],
    pub phases: Vec<Phase>,
    pub groups: Vec<(&'static str, Vec<usize>)>, // 2–3 grupos de botones
    pub sel: Vec<bool>,    // fases seleccionadas (fondo + escalones)
    pub slots: Vec<Slot>,  // estado ON/OFF por intervalo
    pub cooler_from: f64,  // minuto ENTERO en que se enciende el ENFRIADOR (apagado antes, encendido después)
}

// bits (5, de izq. a der.): SOL-BAJA, SOL-ALTA, BY-PASS, CICLON, ENFRIADOR (1 = ON)
fn p(name: &'static str, end: End, b: u8) -> Phase {
    Phase { name, end, v: std::array::from_fn(|k| (b >> (NV - 1 - k)) & 1 == 1) }
}

impl Recipe {
    pub fn curve(&self) -> Pchip { Pchip::new(&self.pts) }
    /// (t_inicio, t_fin) de cada fase; las dos últimas siguen a descarga y fin de enfriamiento.
    pub fn spans(&self) -> Vec<(f64, f64)> {
        let (d, e) = (self.pts[3].0, self.pts[4].0);
        let mut s = 0.0;
        self.phases.iter().map(|ph| {
            let t1 = match ph.end { End::At(x) => x.min(d - 0.1), End::Discharge => d, End::Cooled => e };
            let o = (s, t1); s = t1; o
        }).collect()
    }
    /// (índice, t0, t1, fase) de los intervalos que caen antes del fin del enfriamiento.
    pub fn live(&self) -> Vec<(usize, f64, f64, usize)> {
        let (sp, e) = (self.spans(), self.pts[4].0);
        self.slots.iter().enumerate().filter(|(_, s)| s.t0 < e - 1e-9).map(|(k, s)| {
            let t1 = s.t1.min(e);
            let m = (s.t0 + t1) / 2.0;
            (k, s.t0, t1, sp.iter().position(|a| m >= a.0 && m < a.1).unwrap_or(self.phases.len() - 1))
        }).collect()
    }
    pub fn all() -> Vec<Recipe> {
        use End::*;
        let mut v = vec![
            Recipe {
                title: "RECETA 1 — FLOR DE MONTAÑA", sub: "PERFIL SUAVE + ENFRIAMIENTO · filtro",
                color: "#c084fc", tags: "🌸 Floral|🍋 Cítrico|🍑 Durazno", tmax: 220.0,
                pts: [(0., 27.), (8., 113.), (12., 177.), (17., 189.), (19., 40.)],
                phases: vec![
                    p("Precal.", At(3.0), 0b10010), p("Secado", At(5.0), 0b11010),
                    p("Maillard A", At(8.0), 0b11010), p("Maillard B", At(11.0), 0b11110),
                    p("Pre-Crack", At(13.0), 0b01110), p("1er Crack", At(14.0), 0b01110),
                    p("Desarrollo", Discharge, 0b10110), p("Enfriamiento", Cooled, 0b00111),
                ],
                groups: vec![("Secado", vec![0, 1]), ("Maillard", vec![2, 3]), ("Desarrollo", vec![4, 5, 6, 7])],
                sel: vec![false; 8], slots: vec![], cooler_from: 17.0,
            },
            Recipe {
                title: "RECETA 2 — TIERRA OBSCURA", sub: "PERFIL MEDIO + ENFRIAMIENTO · espresso",
                color: "#34d399", tags: "🍫 Chocolate|🥜 Nuez|🍮 Caramelo", tmax: 230.0,
                pts: [(0., 30.), (5., 105.), (12., 175.), (17., 215.), (20., 40.)],
                phases: vec![
                    p("Precal.", At(3.0), 0b10010), p("Secado", At(6.0), 0b11010),
                    p("Maillard A", At(9.0), 0b11010), p("Maillard B", At(12.0), 0b01110),
                    p("Pre-Crack", At(14.0), 0b01110), p("1er Crack", At(15.0), 0b01110),
                    p("Desarr. A", At(16.0), 0b10110), p("Desarr. B", Discharge, 0b10110),
                    p("Enfriamiento", Cooled, 0b00111),
                ],
                groups: vec![("Secado", vec![0, 1]), ("Maillard", vec![2, 3]), ("Desarrollo", vec![4, 5, 6, 7, 8])],
                sel: vec![false; 9], slots: vec![], cooler_from: 17.0,
            },
        ];
        for r in v.iter_mut() {
            let sp = r.spans();
            let mut cuts: Vec<f64> = (0..=28).map(|k| k as f64).collect();
            cuts.sort_by(|a, b| a.partial_cmp(b).unwrap());
            r.slots = cuts.windows(2).map(|w| {
                let m = (w[0] + w[1]) / 2.0;
                let i = sp.iter().position(|a| m >= a.0 && m < a.1).unwrap_or(r.phases.len() - 1);
                Slot { t0: w[0], t1: w[1], v: r.phases[i].v }
            }).collect();
            let cf = r.cooler_from.round();
            for s in r.slots.iter_mut() { s.v[NV - 1] = s.t0 >= cf - 1e-9; } // ENFRIADOR
        }
        v
    }
}

impl Recipe {
    /// Coloca un punto de control. Tiempos en pasos de 1 min; descarga limitada a 13–25 min
    /// (al mover la descarga el enfriamiento conserva su duración).
    pub fn place(&mut self, p: usize, t: Option<f64>, temp: Option<f64>) {
        if let Some(v) = temp { self.pts[p].1 = v.round().max(25.0).min(self.tmax); }
        if let (Some(t), true) = (t, p > 0) {
            let t = (t / STEP).round() * STEP;
            if p == 3 {
                let gap = (self.pts[4].0 - self.pts[3].0).max(STEP);
                self.pts[3].0 = t.max((self.pts[2].0 + STEP).max(ROAST_MIN)).min(ROAST_MAX);
                self.pts[4].0 = (self.pts[3].0 + gap).min(T_AXIS);
            } else {
                let hi = if p < 4 { self.pts[p + 1].0 - STEP } else { T_AXIS };
                self.pts[p].0 = t.max(self.pts[p - 1].0 + STEP).min(hi);
            }
        }
    }
}

/// 4.5 → "4:30"
pub fn mmss(t: f64) -> String {
    let s = (t * 60.0).round() as i64;
    format!("{}:{:02}", s / 60, s % 60)
}

/// Acepta "m:ss" o minutos decimales ("4,5" / "4.5").
pub fn parse_t(s: &str) -> Option<f64> {
    let s = s.trim().replace(',', ".");
    match s.split_once(':') {
        Some((m, sec)) => Some(m.trim().parse::<f64>().ok()? + sec.trim().parse::<f64>().ok()? / 60.0),
        None => s.parse().ok(),
    }
}
