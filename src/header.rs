pub const HEADER_VERSION: &str = "4U";

/// Insertar UNA sola vez en la página.
pub const HEADER_CSS: &str = r#"
<style>
.ct-header {
    --ct-accent: var(--c1, #22d3ee);
    display: flex; justify-content: space-between; align-items: center;
    border-bottom: 1px solid var(--ln, #1c2128);
    padding: 12px 0; margin-bottom: 20px; flex-wrap: wrap; gap: 12px;
    font-family: 'Share Tech Mono', ui-monospace, Menlo, monospace;
}
.ct-brand { display: flex; align-items: center; gap: 10px; font-size: 18px; letter-spacing: .05em; }
.ct-icon { font-size: 20px; }
.ct-title { color: #e8eaed; font-weight: 700; }
.ct-version { color: #f97316; font-size: .9em; margin-left: 4px; }
.ct-tabs { display: flex; gap: 8px; }
.ct-tab {
    display: flex; align-items: center; gap: 8px; text-decoration: none;
    color: var(--m, #7d8590); font-size: 12px; font-weight: 700;
    letter-spacing: .1em; padding: 6px 12px; border-radius: 6px;
    border: 1px solid transparent;
    transition: color .2s, background .2s, border-color .2s;
}
.ct-tab:hover, .ct-tab[aria-current="page"] {
    color: #fff;
    background: color-mix(in srgb, var(--ct-accent) 5%, transparent);
    border-color: color-mix(in srgb, var(--ct-accent) 20%, transparent);
}
.ct-badge {
    background: var(--ct-accent); color: #000;
    font-family: 'Orbitron', sans-serif; font-size: 11px; font-weight: 800;
    padding: 2px 6px; border-radius: 4px; min-width: 24px; text-align: center;
    box-shadow: 0 0 8px color-mix(in srgb, var(--ct-accent) 40%, transparent);
}
@media (max-width: 560px) {
    .ct-header { flex-direction: column; align-items: flex-start; }
    .ct-tabs, .ct-tab { width: 100%; }
    .ct-tab { justify-content: center; border-color: var(--ln, #1c2128); }
}
</style>
"#;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
     .replace('"', "&quot;").replace('\'', "&#39;")
}

pub fn render_header(number: usize, label: &str) -> String {
    let badge = format!("{number:02}");
    let label = esc(label);
    format!(
        r##"<header class="ct-header">
    <div class="ct-brand">
        <span class="ct-icon" aria-hidden="true">☕</span>
        <span class="ct-title">CoffeeTorrefaction
            <span class="ct-version">{HEADER_VERSION}</span></span>
    </div>
    <nav class="ct-tabs" aria-label="Sección">
        <a href="#r-{number}" class="ct-tab" aria-current="page">
            <span class="ct-badge">{badge}</span>
            <span class="ct-label">{label}</span>
        </a>
    </nav>
</header>"##
    )
}