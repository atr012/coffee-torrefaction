//! CoffeeTorrefaction v4U — núcleo en Rust/WASM.
//! Rust: modelo, matemática (PCHIP), SVG y HTML del panel. JS (www/main.js): solo eventos del DOM.
mod curve;
mod panel;
mod recipe;
mod report;
mod svg;
mod header;

use recipe::*;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub fn header_css() -> String {
    crate::header::HEADER_CSS.to_string()
}

#[wasm_bindgen]
pub fn header(number: usize, label: &str) -> String {
    crate::header::render_header(number, label)
}

#[wasm_bindgen]
pub struct App { cur: Vec<Recipe>, orig: Vec<Recipe> }

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> App { let r = Recipe::all(); App { cur: r.clone(), orig: r } }
    pub fn count(&self) -> usize { self.cur.len() }
    pub fn head(&self, i: usize) -> String {
        let r = &self.cur[i];
        format!("<div class=\"ct-k\">{}</div><div class=\"ct-sub\" style=\"color:{}\">◆ {}</div>", r.title, r.color, r.sub)
    }
    /// mode: "mod" | "orig" | "both" (orig/both dibujan la curva original punteada)
    pub fn svg(&self, i: usize, w: f64, mode: &str) -> String {
        let o = if mode == "mod" { None } else { Some(&self.orig[i].pts) };
        svg::render(&self.cur[i], o, w, false)
    }
    pub fn panel(&self, i: usize) -> String { panel::panel(&self.cur[i]) }
    pub fn equations(&self, i: usize, mode: &str) -> String { panel::equations(&self.cur[i], &self.orig[i], mode) }
    pub fn compare(&self) -> String {
        let t: Vec<String> = self.cur.iter().map(|r| format!("{} min", r.pts[4].0)).collect();
        let d: Vec<String> = self.cur.iter().map(|r| format!("{}°C", r.pts[3].1)).collect();
        format!("<div><small>TIEMPO TOTAL</small>{}</div><div><small>T° DESCARGA</small>{}</div>", t.join("<br>"), d.join("<br>"))
    }
    /// Arrastre de un punto (coordenadas SVG). Eje X fijo 0–28 min, pasos de 30 s.
    pub fn drag(&mut self, i: usize, p: usize, x: f64, y: f64, w: f64) {
        let l = svg::Lay::new(w);
        let r = &mut self.cur[i];
        let tm = r.tmax;
        r.place(p, Some(l.t(x)), Some(l.v(tm, y)));
    }
    /// Editor numérico de tiempos (± 30 s) y temperaturas de los puntos.
    pub fn editor(&self, i: usize) -> String { panel::editor(&self.cur[i]) }
    /// field: "t" (± 30 s) | "T" (± 1 °C)
    pub fn nudge(&mut self, i: usize, p: usize, field: &str, step: i32) {
        let r = &mut self.cur[i];
        if field == "t" { let t = r.pts[p].0 + 0.5 * step as f64; r.place(p, Some(t), None) }
        else { let v = r.pts[p].1 + step as f64; r.place(p, None, Some(v)) }
    }
    /// Tiempo en "m:ss" o minutos decimales; se redondea a 30 s.
    pub fn set_time(&mut self, i: usize, p: usize, s: &str) {
        if let Some(t) = parse_t(s) { self.cur[i].place(p, Some(t), None) }
    }
    pub fn set_temp(&mut self, i: usize, p: usize, v: f64) {
        if v.is_finite() { self.cur[i].place(p, None, Some(v)) }
    }
    /// Conmuta una válvula en un intervalo (k = índice de slot) y selecciona su fase.
    pub fn toggle(&mut self, i: usize, k: usize, v: usize) {
        let r = &mut self.cur[i];
        r.slots[k].v[v] = !r.slots[k].v[v];
        if let Some(e) = r.live().iter().find(|e| e.0 == k) { r.sel[e.3] = true; }
    }
    /// Hoja del reporte (blanco y negro) con gráfica, fórmulas y tablas.
    pub fn report(&self, i: usize) -> String {
        let mut c = self.cur[i].clone();
        c.sel = vec![false; c.sel.len()];
        report::html(&self.cur[i], &self.orig[i], &svg::render(&c, None, 720.0, true))
    }
    /// Todas las tablas en TSV (se pega directo en Excel / Google Sheets).
    pub fn tsv(&self) -> String { self.cur.iter().map(report::tsv).collect::<Vec<_>>().join("\n") }
    /// g = 0: todas las fases; g ≥ 1: solo ese grupo.
    pub fn select_group(&mut self, i: usize, g: usize) {
        let r = &mut self.cur[i];
        for k in 0..r.sel.len() { r.sel[k] = g == 0 || r.groups[g - 1].1.contains(&k); }
    }
    pub fn reset(&mut self, i: usize) { self.cur[i] = self.orig[i].clone(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn humo() {
        let mut a = App::new();
        for i in 0..a.count() {
            a.select_group(i, 2);
            a.toggle(i, 5, 0);
            a.drag(i, 2, 200.0, 150.0, 360.0);
            a.drag(i, 4, 9999.0, 0.0, 360.0);
            let s = a.svg(i, 360.0, "both");
            assert!(s.starts_with("<svg") && s.ends_with("</svg>") && !s.contains("NaN"));
            assert!(a.panel(i).contains("ct-t") && a.equations(i, "both").contains("T4(t)"));
            a.reset(i);
            assert!(a.equations(i, "both").contains("ORIGINALES = MODIFICADOS"));
        }
        let r = Recipe::all();
        let n = |ph: usize| r[0].live().iter().filter(|e| e.3 == ph).count();
        assert_eq!((n(0), n(2)), (3, 3)); // Precal 0–2.5 → 3 intervalos; Maillard A 5–8 → 3
        assert!(a.report(0).contains("<table") && a.tsv().contains("SOL-BAJA\tSOL-ALTA"));
        assert!(a.compare().contains("18 min"));
    }

    #[test]
    fn tiempos_30s_y_limites() {
        assert_eq!((parse_t("4:30"), parse_t("4,7")), (Some(4.5), Some(4.7)));
        let mut a = App::new();
        a.set_time(0, 2, "10:20"); // → 10:30
        assert_eq!(a.cur[0].pts[2].0, 10.5);
        a.set_time(0, 3, "40:00"); // descarga tope 25 min; enfriamiento conserva 3 min
        assert_eq!((a.cur[0].pts[3].0, a.cur[0].pts[4].0), (25.0, 28.0));
        a.set_time(0, 3, "5:00");
        assert!(a.cur[0].pts[3].0 >= ROAST_MIN);
        a.nudge(0, 1, "t", 1);
        assert!(a.editor(0).contains("ct-er") && mmss(15.0) == "15:00");
    }
}

