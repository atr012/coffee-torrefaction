//! Render SVG (responsivo: el ancho en px viene del contenedor, el texto no se encoge).
use crate::{curve::Pchip, recipe::*};
use std::fmt::Write;
macro_rules! w { ($s:expr, $($a:tt)*) => { write!($s, $($a)*).unwrap() } }
/// Escapa texto antes de meterlo en HTML/SVG (evita que "<" o "\"" abran etiquetas o atributos).
/// El & va primero a propósito; si lo pones después, escapas dos veces.
pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('"', "&quot;").replace('\'', "&#39;")
}

pub const COL: [&str; 4] = ["#fbbf24", "#fb923c", "#22d3ee", "#e5e7eb"];
const DASH: [&str; 4] = ["", "7 4", "2 3", "9 3 2 3"];
const BAND: [&str; 9] = ["#0e7490", "#0891b2", "#854d0e", "#a16207", "#c2410c", "#b91c1c", "#6d28d9", "#7c3aed", "#475569"];

pub struct Lay { pub w: f64, pub ml: f64, pub mr: f64, pub mt: f64, pub ph: f64 }
impl Lay {
    pub fn new(w: f64) -> Self { Lay { w, ml: 50.0, mr: 12.0, mt: 22.0, ph: (w * 0.55).clamp(240.0, 380.0) } }
    pub fn x(&self, t: f64) -> f64 { self.ml + t / T_AXIS * (self.w - self.ml - self.mr) }
    pub fn t(&self, x: f64) -> f64 { (x - self.ml) / (self.w - self.ml - self.mr) * T_AXIS }
    pub fn y(&self, tmax: f64, v: f64) -> f64 { self.mt + self.ph - (v - 20.0) / (tmax - 20.0) * self.ph }
    pub fn v(&self, tmax: f64, y: f64) -> f64 { 20.0 + (self.mt + self.ph - y) / self.ph * (tmax - 20.0) }
    pub fn h(&self) -> f64 { self.mt + self.ph + 42.0 + 4.0 * 20.0 + 6.0 }
}

fn path(l: &Lay, tmax: f64, f: &Pchip, tend: f64) -> String {
    let mut d = String::new();
    for i in 0..=120 {
        let t = tend * i as f64 / 120.0;
        w!(d, "{}{:.1} {:.1}", if i == 0 { 'M' } else { 'L' }, l.x(t), l.y(tmax, f.at(t)));
    }
    d
}

pub fn render(r: &Recipe, orig: Option<&[(f64, f64); 5]>, w: f64, bw: bool) -> String {
    let l = Lay::new(w);
    let col = if bw { "#000" } else { r.color };
    let (f, sp, tm) = (r.curve(), r.spans(), r.tmax);
    let (n, pb) = (sp.len(), l.mt + l.ph);
    let mut s = String::new();
    w!(s, "<svg viewBox=\"0 0 {:.0} {:.0}\" width=\"100%\" role=\"img\" aria-label=\"{}\">", w, l.h(), r.title);
    // ventana del tostador cilíndrico: mín 13, máx 25 min, óptimo 15–17 min
    let gc = if bw { "#000" } else { "#22c55e" };
    w!(s, "<rect x=\"{:.1}\" y=\"{}\" width=\"{:.1}\" height=\"{}\" fill=\"{}\" opacity=\".09\" stroke=\"{}\" stroke-dasharray=\"2 3\"/>", l.x(BEST.0), l.mt, l.x(BEST.1) - l.x(BEST.0), l.ph, gc, gc);
    w!(s, "<text class=\"bl\" style=\"fill:{}\" text-anchor=\"middle\" x=\"{:.1}\" y=\"{}\">ÓPTIMO 15–17</text>", gc, l.x((BEST.0 + BEST.1) / 2.0), l.mt - 8.0);
    for (t, lab) in [(ROAST_MIN, "MÍN 13"), (ROAST_MAX, "MÁX 25")] {
        w!(s, "<line x1=\"{x:.1}\" x2=\"{x:.1}\" y1=\"{}\" y2=\"{}\" stroke=\"#6b7280\" stroke-dasharray=\"6 4\"/>", l.mt, pb, x = l.x(t));
        w!(s, "<text class=\"bl\" text-anchor=\"end\" x=\"{:.1}\" y=\"{}\">{}</text>", l.x(t) - 2.0, l.mt - 8.0, lab);
    }
    // bandas de fase
    for (i, (a, b)) in sp.iter().enumerate() {
        let (x0, ww) = (l.x(*a), (l.x(*b) - l.x(*a)).max(0.0));
        let c = if i == n - 1 { BAND[8] } else { BAND[i.min(7)] };
        let (c, op) = if bw { ("#000", if i % 2 == 0 { ".05" } else { ".11" }) } else { (c, ".16") };
        w!(s, "<rect x=\"{:.1}\" y=\"{}\" width=\"{:.1}\" height=\"{}\" fill=\"{}\" opacity=\"{}\"/>", x0, l.mt, ww, l.ph, c, op);
        if ww > 44.0 { w!(s, "<text class=\"bl\" x=\"{:.1}\" y=\"{}\">{}</text>", x0 + 3.0, l.mt + 11.0, r.phases[i].name.to_uppercase()); }
    }
    // rejilla Y
    for g in 0..=4 {
        let y = l.y(tm, 20.0 + g as f64 * (tm - 20.0) / 4.0);
        w!(s, "<line x1=\"{}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" class=\"gl\"/>", l.ml, w - l.mr, y, y);
        w!(s, "<text class=\"tk\" text-anchor=\"end\" x=\"{}\" y=\"{:.1}\">{:.0}°</text>", l.ml - 6.0, y + 3.0, 20.0 + g as f64 * (tm - 20.0) / 4.0);
    }
    // eje de minutos FIJO en la parte inferior
    let step = if w > 760.0 { 1 } else if w > 420.0 { 2 } else { 4 };
    for m in (0..=28).step_by(step) {
        let x = l.x(m as f64);
        w!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{}\" y2=\"{}\" class=\"gl\"/>", x, x, l.mt, pb);
        w!(s, "<text class=\"tk\" text-anchor=\"middle\" x=\"{:.1}\" y=\"{}\">{}</text>", x, pb + 14.0, m);
    }
    w!(s, "<text class=\"tk\" text-anchor=\"end\" x=\"{}\" y=\"{}\">min</text>", l.ml - 8.0, pb + 14.0);
    // fases seleccionadas: fondo de otro color + borde punteado
    for (i, (a, b)) in sp.iter().enumerate().filter(|(i, _)| r.sel[*i]) {
        let _ = i;
        w!(s, "<rect x=\"{:.1}\" y=\"{}\" width=\"{:.1}\" height=\"{}\" fill=\"{c}\" opacity=\".3\" stroke=\"{c}\" stroke-dasharray=\"5 3\"/>",
            l.x(*a), l.mt, (l.x(*b) - l.x(*a)).max(0.0), l.ph, c = r.color);
    }
    // curva original (referencia)
    if let Some(o) = orig.filter(|o| **o != r.pts) {
        w!(s, "<path d=\"{}\" fill=\"none\" stroke=\"#9ca3af\" stroke-width=\"1.5\" stroke-dasharray=\"3 4\"/>", path(&l, tm, &Pchip::new(o), o[4].0));
    }
    let yd = l.y(tm, r.pts[3].1);
    w!(s, "<line x1=\"{}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-dasharray=\"4 4\" opacity=\".5\"/>", l.ml, w - l.mr, yd, yd, col);
    w!(s, "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"3\" stroke-linecap=\"round\"/>", path(&l, tm, &f, r.pts[4].0), col);
    // 1er crack
    if let Some(ci) = r.phases.iter().position(|p| p.name == "1er Crack") {
        let t = sp[ci].1;
        w!(s, "<line x1=\"{x:.1}\" x2=\"{x:.1}\" y1=\"{:.1}\" y2=\"{}\" stroke=\"#fb923c\" stroke-dasharray=\"2 3\"/>", l.y(tm, f.at(t)), pb, x = l.x(t));
        w!(s, "<text class=\"bl\" style=\"fill:#fb923c\" x=\"{:.1}\" y=\"{}\">1er crack ≈{:.0}°</text>", l.x(t) + 4.0, l.mt + 22.0, f.at(t));
    }
    // escalones ON/OFF por intervalo (solo fases seleccionadas); un estilo de línea y color por válvula
    let live = r.live();
    for j in 0..4 {
        let (base, hi) = (pb - 6.0 - j as f64 * 16.0, pb - 16.0 - j as f64 * 16.0);
        let (mut d, mut last) = (String::new(), None::<usize>);
        for (n_, &(k, t0, t1, ph)) in live.iter().enumerate().filter(|(_, e)| r.sel[e.3]) {
            let y = if r.slots[k].v[j] { hi } else { base };
            let cmd = if n_ > 0 && last == Some(n_ - 1) { 'L' } else { 'M' };
            w!(d, "{}{:.1} {:.1} L{:.1} {:.1} ", cmd, l.x(t0), y, l.x(t1), y);
            if last.is_none() { w!(s, "<text class=\"bl\" style=\"fill:{}\" x=\"{:.1}\" y=\"{:.1}\">{}</text>", COL[j], l.x(t0) + 3.0, y - 3.0, VALVES[j]); }
            last = Some(n_);
            let _ = ph;
        }
        if !d.is_empty() { w!(s, "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\" stroke-dasharray=\"{}\"/>", d, COL[j], DASH[j]); }
    }
    // líneas guía y tiempo mm:ss al pie, alineado con cada temperatura
    for &(t, v) in r.pts.iter() {
        let (cx, cy) = (l.x(t), l.y(tm, v));
        w!(s, "<line x1=\"{x:.1}\" x2=\"{x:.1}\" y1=\"{:.1}\" y2=\"{}\" stroke=\"{}\" stroke-dasharray=\"1 3\" opacity=\".7\"/>", cy, pb, col, x = cx);
        let bx = (cx - 17.0).max(l.ml - 2.0).min(w - l.mr - 34.0);
        w!(s, "<rect x=\"{:.1}\" y=\"{}\" width=\"34\" height=\"14\" rx=\"3\" fill=\"{}\" stroke=\"{}\"/>", bx, pb + 20.0, if bw { "#fff" } else { "#000" }, col);
        w!(s, "<text class=\"tt\" text-anchor=\"middle\" x=\"{:.1}\" y=\"{}\">{}</text>", bx + 17.0, pb + 31.0, mmss(t));
    }
    // puntos de control (zona táctil de 20 px de radio)
    for (i, &(t, v)) in r.pts.iter().enumerate() {
        let (cx, cy) = (l.x(t), l.y(tm, v));
        let (anc, dx) = if i == 4 || cx > w - 70.0 { ("end", -10.0) } else { ("start", 10.0) };
        w!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"7\" fill=\"{}\" stroke=\"{}\" stroke-width=\"3\"/>", cx, cy, if bw { "#fff" } else { "#000" }, col);
        w!(s, "<text class=\"pl\" text-anchor=\"{}\" x=\"{:.1}\" y=\"{:.1}\">{:.0}°C</text>", anc, cx + dx, cy - 10.0, v);
        w!(s, "<circle class=\"pt\" data-p=\"{}\" cx=\"{:.1}\" cy=\"{:.1}\" r=\"20\" fill=\"transparent\"/>", i, cx, cy);
    }
    // línea de tiempo de válvulas: una celda por intervalo (tocar la celda la conmuta)
    for j in 0..4 {
        let y0 = pb + 42.0 + j as f64 * 20.0;
        w!(s, "<text class=\"bl\" text-anchor=\"end\" x=\"{}\" y=\"{}\">{}</text>", l.ml - 4.0, y0 + 12.0, VALVES[j]);
        for &(k, t0, t1, _) in &live {
            let (x0, ww, on) = (l.x(t0), (l.x(t1) - l.x(t0)).max(0.0), r.slots[k].v[j]);
            let (fill, op) = if bw { (if on { "#444" } else { "#eee" }, "1") } else if on { (r.color, ".55") } else { ("#0d1117", "1") };
            w!(s, "<rect class=\"vc\" data-k=\"{}\" data-v=\"{}\" x=\"{:.1}\" y=\"{}\" width=\"{:.1}\" height=\"17\" rx=\"3\" fill=\"{}\" opacity=\"{}\" stroke=\"{}\"/>",
                k, j, x0 + 0.5, y0, (ww - 1.0).max(0.0), fill, op, if bw { "#999" } else { "#21262d" });
            if ww > 22.0 { w!(s, "<text class=\"vt\" style=\"fill:{}\" x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\">{}</text>", if bw && !on { "#000" } else { "#fff" }, x0 + ww / 2.0, y0 + 12.0, if on { "ON" } else { "OFF" }); }
        }
    }
    s.push_str("</svg>");
    s
}
