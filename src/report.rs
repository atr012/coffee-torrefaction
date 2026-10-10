//! Reporte: fórmulas + tablas (HTML para imprimir, TSV para pegar en una hoja de cálculo).
use crate::{curve::Pchip, panel, recipe::*};
use std::fmt::Write;

type Sheet = (String, Vec<String>, Vec<Vec<String>>);
fn row<const N: usize>(c: [String; N]) -> Vec<String> { c.to_vec() }

pub fn sheets(r: &Recipe) -> Vec<Sheet> {
    let (f, sp) = (r.curve(), r.spans());
    let nm = ["Carga", "Punto 1", "Punto 2", "Descarga", "Fin enfriamiento"];
    let h = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    vec![
        ("Puntos de control".into(), h(&["Punto", "t (min)", "t (mm:ss)", "T (°C)"]),
         r.pts.iter().enumerate().map(|(i, p)| row([nm[i].into(), p.0.to_string(), mmss(p.0), p.1.to_string()])).collect()),
        ("Polinomios por tramo: T(t) = a + b·(t−t0) + c·(t−t0)^2 + d·(t−t0)^3".into(), h(&["Tramo", "t0 (min)", "t1 (min)", "a", "b", "c", "d"]),
         Pchip::new(&r.pts).segs.iter().enumerate().map(|(i, g)| row([format!("T{}", i + 1), g.t0.to_string(), g.t1.to_string(),
             format!("{:.6}", g.a), format!("{:.6}", g.b), format!("{:.6}", g.c), format!("{:.6}", g.d)])).collect()),
        ("Fases".into(), h(&["Fase", "t inicio (mm:ss)", "t fin (mm:ss)", "t inicio (min)", "t fin (min)", "T SP (°C)"]),
         r.phases.iter().enumerate().map(|(i, p)| row([p.name.into(), mmss(sp[i].0), mmss(sp[i].1), sp[i].0.to_string(), sp[i].1.to_string(), format!("{:.1}", f.at(sp[i].1))])).collect()),
        ("Válvulas por intervalo (1 = ON, 0 = OFF)".into(), h(&["t inicio (mm:ss)", "t fin (mm:ss)", "t inicio (min)", "t fin (min)", "Fase", "T inicio (°C)", "T fin (°C)", "SOL-BAJA", "SOL-ALTA", "BY-PASS", "CICLON","ENFRIADOR"]),
         r.live().iter().map(|&(k, t0, t1, ph)| {
             let v = r.slots[k].v.map(|b| (b as u8).to_string());
             row([mmss(t0), mmss(t1), t0.to_string(), t1.to_string(), r.phases[ph].name.into(), format!("{:.1}", f.at(t0)), format!("{:.1}", f.at(t1)),
                  v[0].clone(), v[1].clone(), v[2].clone(), v[3].clone()])
         }).collect()),
    ]
}

pub fn html(r: &Recipe, o: &Recipe, svg: &str) -> String {
    let mut s = format!("<article class=\"ct-sheet\"><h2>{}</h2><p>{}</p>{}<h3>Fórmulas</h3><pre>{}</pre>", r.title, r.sub, svg, panel::equations(r, o, "both"));
    for (t, head, rows) in sheets(r) {
        write!(s, "<h3>{}</h3><table class=\"ct-ss\"><thead><tr>", t).unwrap();
        for c in &head { write!(s, "<th>{}</th>", c).unwrap(); }
        s.push_str("</tr></thead><tbody>");
        for rw in rows { s.push_str("<tr>"); for c in rw { write!(s, "<td>{}</td>", c).unwrap(); } s.push_str("</tr>"); }
        s.push_str("</tbody></table>");
    }
    s.push_str("</article>");
    s
}

pub fn tsv(r: &Recipe) -> String {
    let mut s = format!("{}\n\n", r.title);
    for (t, head, rows) in sheets(r) {
        writeln!(s, "{}\n{}", t, head.join("\t")).unwrap();
        for rw in rows { writeln!(s, "{}", rw.join("\t")).unwrap(); }
        s.push('\n');
    }
    s
}
