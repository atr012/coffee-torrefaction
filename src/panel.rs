//! HTML del panel táctil (grupos de botones + rejilla ON/OFF) y ecuaciones.
///use crate::{curve::Pchip, recipe::*, svg::COL};
use crate::{curve::Pchip, recipe::*, svg::{COL, esc}};
use std::fmt::Write;
macro_rules! w { ($s:expr, $($a:tt)*) => { write!($s, $($a)*).unwrap() } }

pub fn panel(r: &Recipe) -> String {
    let (sp, f) = (r.spans(), r.curve());
    let all = r.sel.iter().all(|&b| b);
    let mut s = String::from("<div class=\"ct-tabs\">");
    w!(s, "<button class=\"ct-g{}\" data-g=\"0\">Todas</button>", if all { " a" } else { "" });
    for (k, (name, idx)) in r.groups.iter().enumerate() {
        let act = !all && (0..r.sel.len()).all(|i| r.sel[i] == idx.contains(&i));
        w!(s, "<button class=\"ct-g{}\" data-g=\"{}\">{}</button>", if act { " a" } else { "" }, k + 1, esc(name));
    }
    s.push_str("</div>");
    let live = r.live();
    let vis: Vec<&(usize, f64, f64, usize)> = live.iter().filter(|e| r.sel[e.3]).collect();
    if vis.is_empty() {
        s.push_str("<p class=\"ct-hint\">Elige un grupo para ver sus escalones ON/OFF en la gráfica y conmutarlos (un botón por minuto).</p>");
    } else {
        w!(s, "<div class=\"ct-scroll\"><div class=\"ct-grid\" style=\"grid-template-columns:84px repeat({},minmax(46px,1fr))\"><span></span>", vis.len());
        let mut i = 0;
        while i < vis.len() {
            let ph = vis[i].3;
            let mut j = i;
            while j < vis.len() && vis[j].3 == ph { j += 1; }
            w!(s, "<b style=\"grid-column:span {}\">{}</b>", j - i, r.phases[ph].name);
            i = j;
        }
        s.push_str("<span></span>");
        for e in &vis { w!(s, "<i>{}–{}</i>", e.1, e.2); }
        for j in 0..4 {
            w!(s, "<em style=\"color:{}\">{}</em>", COL[j], VALVES[j]);
            for e in &vis {
                let on = r.slots[e.0].v[j];
                w!(s, "<button class=\"ct-t {}\" style=\"--c:{}\" data-k=\"{}\" data-v=\"{}\">{}</button>", if on { "on" } else { "off" }, COL[j], e.0, j, if on { "ON" } else { "OFF" });
            }
        }
        s.push_str("</div></div>");
    }
    s.push_str("<table class=\"ct-tb\"><tr><th>Fase<th>min<th>T SP</tr>");
    for (i, ph) in r.phases.iter().enumerate() { w!(s, "<tr><td>{}<td>{}–{}<td>{:.0}°C</tr>", esc(&ph.name), sp[i].0, sp[i].1, f.at(sp[i].1)); }
    s.push_str("</table><div class=\"ct-tags\">");
    for t in r.tags.split('|') { w!(s, "<span>{}</span>", esc(t)); }
    s.push_str("</div>");
    s
}

fn sg(v: f64) -> String { format!("{} {:.4}", if v < 0.0 { '−' } else { '+' }, v.abs()) }

fn eq(pts: &[(f64, f64); 5], tag: &str) -> String {
    let mut s = format!("{}\n", tag);
    for (k, g) in Pchip::new(pts).segs.iter().enumerate() {
        w!(s, "T{}(t) = {:.3} {}τ {}τ² {}τ³   τ = t − {}   [{}, {}] min\n", k + 1, g.a, sg(g.b), sg(g.c), sg(g.d), g.t0, g.t0, g.t1);
    }
    s
}

pub fn equations(r: &Recipe, o: &Recipe, mode: &str) -> String {
    let mut s = String::from("T(t) = a + b·τ + c·τ² + d·τ³   (°C, min)\n\n");
    let same = r.pts == o.pts;
    if mode == "orig" || (mode == "both" && !same) { s += &eq(&o.pts, if same { "ORIGINALES = MODIFICADOS" } else { "ORIGINALES" }); s.push('\n'); }
    if mode == "mod" || (mode == "both" && !same) { s += &eq(&r.pts, "MODIFICADOS"); }
    if mode == "both" && same { s += &eq(&r.pts, "ORIGINALES = MODIFICADOS"); }
    s
}

/// Editor de tiempos (pasos de 30 s) y temperaturas de los 5 puntos de control.
pub fn editor(r: &Recipe) -> String {
    let names = ["Carga", "Punto 1", "Punto 2", "Descarga", "Fin enfriam."];
    let d = r.pts[3].0;
    let msg = if d < BEST.0 { "tueste rápido (13–15 min)" } else if d <= BEST.1 { "dentro del rango óptimo (15–17 min)" } else { "tueste largo / oscuro (17–25 min)" };
    let mut s = format!("<div class=\"ct-eh\">⏱ TIEMPOS Y TEMPERATURAS</div><p class=\"ct-hint\">Descarga a los {} — {}. Límites del tostador: {}–{} min.</p>", mmss(d), msg, ROAST_MIN, ROAST_MAX);
    s.push_str("<div class=\"ct-er ct-lg\"><span></span><small>TIEMPO m:ss (± 30 s)</small><small>TEMPERATURA °C (± 1)</small></div>");
    for (p, &(t, v)) in r.pts.iter().enumerate() {
        let dis = if p == 0 { " disabled" } else { "" };
        w!(s, "<div class=\"ct-er\"><span>{}</span><div class=\"ct-st\"><button data-n=\"t\" data-p=\"{p}\" data-s=\"-1\"{dis}>−</button><input data-f=\"t\" data-p=\"{p}\" value=\"{}\" inputmode=\"numeric\" aria-label=\"tiempo mm:ss\"{dis}><button data-n=\"t\" data-p=\"{p}\" data-s=\"1\"{dis}>+</button></div><div class=\"ct-st\"><button data-n=\"T\" data-p=\"{p}\" data-s=\"-1\">−</button><input data-f=\"T\" data-p=\"{p}\" type=\"text\" inputmode=\"numeric\" value=\"{}\" aria-label=\"temperatura\"><button data-n=\"T\" data-p=\"{p}\" data-s=\"1\">+</button></div></div>", names[p], mmss(t), v, p = p, dis = dis);
    }
    s
}
