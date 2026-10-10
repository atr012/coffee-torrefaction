/* ============================================================================
   main.js  —  "El mesero" de la aplicación CoffeeTorrefaction
   ============================================================================

   ¿QUÉ HACE ESTE ARCHIVO?
   Imagina un restaurante:
     - Rust (el archivo .wasm) es la COCINA: ahí se hacen todos los cálculos,
       la curva de tueste, los dibujos y los textos.
     - Este archivo (main.js) es el MESERO: escucha lo que pide el cliente
       (clics, arrastres, teclas), se lo avisa a la cocina, y luego pone en la
       pantalla lo que la cocina le entrega.

   Este archivo NO calcula nada de café. Solo conecta lo que haces con la
   pantalla (clic → Rust → dibujo nuevo).

   CÓMO SE USA EN UNA PÁGINA WEB:
     <div id="ct-root"></div>
     <script type="module">
       import {mount} from '/proyectos/coffee-torrefaction/main.js';
       mount(document.getElementById('ct-root'))
     </script>
   ============================================================================ */


// Traemos la "cocina" (Rust compilado a WebAssembly).
//  - init : función que enciende la cocina
//  - App  : el programa de Rust, con todas sus funciones (svg, editor, drag...)
import init, { App, header, header_css } from './pkg/coffee_torrefaction.js';


// "mount" significa "montar": construye toda la aplicación dentro de un
// contenedor de la página (root = el <div id="ct-root">).
export async function mount(root) {

  // Esperamos a que la cocina esté lista antes de pedir nada.
  await init();

  // Creamos la aplicación y guardamos tres cosas:
  //   app  = la aplicación de Rust (aquí están los datos de las recetas)
  //   n    = cuántas recetas hay (ahora son 2)
  //   mode = qué curva se ve en cada receta: 'mod' (modificada),
  //          'orig' (original) o 'both' (ambas). Empieza en 'mod' para todas.
  const app = new App(), n = app.count(), mode = Array(n).fill('mod');

  // Le ponemos la clase "ct" al contenedor para que el CSS (style.css)
  // le dé el estilo oscuro de la aplicación.
  root.classList.add('ct');

  // ---- ARMAR EL ESQUELETO DE LA PÁGINA -------------------------------------
  // Aquí se escribe el HTML vacío de cada receta: unas "cajas" donde después
  // se meterán la gráfica, el editor de números y el panel de botones.
  // (Todavía no hay datos; solo los huecos.)
  //root.innerHTML = Array.from({ length: n }, (_, i) => `<section data-r="${i}">
  root.innerHTML = header_css() + header(1, "RECETAS") + Array.from({ length: n }, (_, i) => `<section data-r="${i}" id="r-${i + 1}">
    ${app.head(i)}<div class="ct-chart"></div><div class="ct-editor"></div><div class="ct-panel"></div>
    <details><summary>ƒ ECUACIÓN</summary><div class="ct-modes">
      <button class="ct-m" data-m="orig">Originales</button><button class="ct-m" data-m="mod">Modificados</button><button class="ct-m" data-m="both">Ambas</button></div><pre></pre></details>
    <button class="ct-rs">↺ Restaurar</button></section>`).join('')
    + `<section><div class="ct-k">COMPARATIVA</div><div class="ct-cmp"></div></section>
    <div class="ct-act"><button class="ct-pr">📄 Reporte (B/N, varias hojas)</button><a href="https://ko-fi.com/albertotr" target="_blank" rel="noopener noreferrer">☕ Apoyar en Ko-fi</a></div><div class="ct-rep"></div>`;

  // Guardamos "atajos" a las cajas importantes para no buscarlas cada vez:
  //   secs = las secciones de cada receta
  //   cmp  = la caja de la comparativa
  //   rep  = la caja del reporte para imprimir
  const secs = [...root.querySelectorAll('section[data-r]')],
        cmp = root.querySelector('.ct-cmp'),
        rep = root.querySelector('.ct-rep');


  // ==========================================================================
  //  draw  —  "DIBUJAR": la función más importante de este archivo
  // ==========================================================================
  // Le pide a Rust la información ACTUAL de la receta número "i" y la pone en
  // la pantalla: gráfica, números, botones y ecuaciones.
  //
  //   i     = número de receta (0 = Flor de Montaña, 1 = Tierra Obscura)
  //   force = ¿obligar a refrescar los números del editor?
  //           true  (por defecto) → SIEMPRE actualiza los números.
  //           false → NO los actualiza si estás escribiendo en una casilla
  //                   (así no se te borra lo que tecleas).
  const draw = (i, force = true) => {
    const s = secs[i];

    // Medimos el ancho disponible en pantalla para que la gráfica se adapte
    // (mínimo 300 px). Funciona igual en celular que en computadora.
    const w = Math.max(300, Math.round(s.clientWidth - 24));

    // 1) La GRÁFICA: Rust devuelve un dibujo (SVG) y lo pegamos en su caja.
    s.querySelector('.ct-chart').innerHTML = app.svg(i, w, mode[i]);

    // 2) El EDITOR DE NÚMEROS (tiempos y temperaturas).
    const ed = s.querySelector('.ct-editor');
    // "a" es la casilla donde está el cursor en este momento (si hay alguna).
    const a = document.activeElement;
    // Actualizamos los números si "force" es true, o si NO estás escribiendo
    // dentro del editor.
    if (force || !(a && a.tagName === 'INPUT' && ed.contains(a))) ed.innerHTML = app.editor(i);

    // 3) El PANEL de botones ON/OFF de las válvulas.
    s.querySelector('.ct-panel').innerHTML = app.panel(i);

    // 4) Resaltamos el botón del modo activo (Originales / Modificados / Ambas).
    s.querySelectorAll('.ct-m').forEach(b => b.classList.toggle('a', b.dataset.m === mode[i]));

    // 5) Las ECUACIONES: solo se calculan si el menú "ƒ ECUACIÓN" está abierto.
    if (s.querySelector('details').open) s.querySelector('pre').textContent = app.equations(i, mode[i]);

    // 6) La COMPARATIVA de tiempos y temperaturas entre las recetas.
    cmp.innerHTML = app.compare();
  };

  // "all" = dibujar TODAS las recetas. Se usa al inicio y al cambiar el tamaño
  // de la ventana. Aquí "false" protege lo que estés escribiendo en una casilla.
  const all = () => secs.forEach((_, i) => draw(i, false));

  // Primer dibujo al abrir la página.
  all();


  // ==========================================================================
  //  ESCUCHAR EVENTOS  —  "el mesero atiende al cliente"
  // ==========================================================================

  // --- Abrir/cerrar el menú de ecuaciones ------------------------------------
  // Cuando abres o cierras "ƒ ECUACIÓN", redibujamos esa receta para mostrar
  // (u ocultar) las ecuaciones.
  root.addEventListener('toggle', e => {
    const s = e.target.closest('section[data-r]');
    if (s) draw(+s.dataset.r, false);
  }, true);


  // --- CLICS (casi todos los botones pasan por aquí) -------------------------
  root.addEventListener('click', e => {

    // "x" será distinto de vacío si se tocó un botón del reporte
    // (volver, imprimir, copiar, descargar).
    const x = e.target.closest('[data-x]');

    // A) Botón "Reporte": arma las hojas en blanco y negro para imprimir.
    if (e.target.closest('.ct-pr')) {
      rep.innerHTML = `<div class="ct-ract"><button data-x="back">← Volver</button><button data-x="print">🖨 Imprimir / PDF</button><button data-x="copy">📋 Copiar tablas (hoja de cálculo)</button><button data-x="csv">⬇ Descargar CSV</button></div>` + secs.map((_, i) => app.report(i)).join('');
      root.classList.add('rep');   // el CSS oculta todo y muestra solo el reporte
      return scrollTo(0, 0);       // sube al inicio de la página
    }

    // B) Botones DEL REPORTE
    if (x) {
      const k = x.dataset.x;       // cuál botón fue

      // Volver a la aplicación normal.
      if (k === 'back') root.classList.remove('rep');

      // Abrir el diálogo de impresión del navegador (o guardar como PDF).
      else if (k === 'print') print();

      // Copiar las tablas para pegarlas en Excel o Google Sheets.
      else if (k === 'copy') {
        const t = app.tsv();       // Rust entrega las tablas como texto
        // Intentamos copiar de la forma moderna; si el navegador no lo permite,
        // usamos un truco antiguo (una caja de texto invisible).
        (navigator.clipboard ? navigator.clipboard.writeText(t) : Promise.reject()).catch(() => {
          const a = document.createElement('textarea');
          a.value = t; document.body.append(a); a.select();
          document.execCommand('copy'); a.remove();
        });
        // Avisamos al usuario con el texto del botón y lo regresamos a normal.
        x.textContent = '✔ Copiado';
        setTimeout(() => x.textContent = '📋 Copiar tablas (hoja de cálculo)', 1500);
      }

      // Descargar las tablas como archivo .csv (se abre en Excel).
      else if (k === 'csv') {
        const a = document.createElement('a');
        a.href = URL.createObjectURL(new Blob(['\ufeff' + app.tsv().replace(/\t/g, ',')], { type: 'text/csv' }));
        a.download = 'coffee-torrefaction.csv';
        a.click();
      }
      return; // ya terminamos con este clic
    }

    // C) Botones DE LA APLICACIÓN (grupos, ON/OFF, modos, ±, restaurar)
    // "s" = la receta donde se hizo clic; "b" = el botón tocado.
    const s = e.target.closest('section[data-r]'),
          b = e.target.closest('[data-g],[data-v],[data-m],[data-n],.ct-rs');
    if (!s || !b) return;          // si no fue un botón nuestro, ignorar
    const i = +s.dataset.r;        // número de receta

    // Cada botón trae una "etiqueta" (data-...) que dice qué hace:
    if (b.dataset.g !== undefined)      app.select_group(i, +b.dataset.g);   // botón de grupo (Secado, Maillard...)
    else if (b.dataset.v !== undefined) app.toggle(i, +b.dataset.k, +b.dataset.v); // botón ON/OFF de una válvula
    else if (b.dataset.n)               app.nudge(i, +b.dataset.p, b.dataset.n, +b.dataset.s); // botones − y +
    else if (b.dataset.m)               mode[i] = b.dataset.m;               // cambiar modo de curva
    else                                app.reset(i);                        // ↺ Restaurar valores originales

    // Después de cualquier cambio, redibujamos. Con force = true (por defecto)
    // los números del editor SIEMPRE se actualizan. Esto arregla tu problema.
    draw(i);
  });


  // --- Evitar que los botones − / + te quiten el cursor de la casilla ---------
  root.addEventListener('mousedown', e => { if (e.target.closest('[data-n]')) e.preventDefault(); });


  // --- ESCRIBIR UN NUMERO A MANO en una casilla ------------------------------
  // "change" ocurre cuando terminas de escribir y sales de la casilla (Enter o clic fuera).
  root.addEventListener('change', e => {
    const f = e.target.dataset.f;  // 't' = casilla de tiempo, 'T' = casilla de temperatura
    if (!f) return;                // si no es una casilla del editor, ignorar
    const i = +e.target.closest('section').dataset.r,  // número de receta
          p = +e.target.dataset.p;                     // número de punto (0 a 4)

    // Seguro: si el valor no cambió, no hacer nada.
    // (Hoy no funciona porque las casillas no traen "data-v"; es inofensivo.)
    if (e.target.dataset.v === e.target.value) return;

    // Tiempo → Rust lo entiende como "4:30" o "4.5"; temperatura → número.
    f === 't' ? app.set_time(i, p, e.target.value) : app.set_temp(i, p, parseFloat(e.target.value));
    draw(i, true);                 // mostrar el valor ya corregido por Rust
  });


  // --- ARRASTRAR LOS PUNTOS DE LA GRÁFICA ------------------------------------
  // Funciona con ratón y con el dedo. Se activa al TOCAR un punto (círculo).
  root.addEventListener('pointerdown', e => {
    const pt = e.target.closest('.pt');
    if (!pt) return;               // si no tocaste un punto, ignorar
    e.preventDefault();            // evita que la página se desplace mientras arrastras

    // Datos del punto: receta (i), cuál punto es (p) y su dibujo (svg).
    const i = +pt.closest('section').dataset.r,
          p = +pt.dataset.p,
          svg = pt.ownerSVGElement;

    // Para convertir la posición del dedo (píxeles de pantalla) a la posición
    // dentro del dibujo, calculamos una escala "k".
    const w = svg.viewBox.baseVal.width,
          b = svg.getBoundingClientRect(),
          k = w / b.width;

    // "mv" se ejecuta cada vez que mueves el dedo/ratón:
    // le dice a Rust la nueva posición y redibuja todo al instante.
    const mv = ev => {
      app.drag(i, p, (ev.clientX - b.left) * k, (ev.clientY - b.top) * k, w);
      draw(i, true);
    };

    // "up" se ejecuta al soltar: deja de escuchar el movimiento (limpieza).
    const up = () => {
      removeEventListener('pointermove', mv);
      removeEventListener('pointerup', up);
      removeEventListener('pointercancel', up);
    };

    // Empezamos a escuchar el movimiento y el soltar.
    addEventListener('pointermove', mv);
    addEventListener('pointerup', up);
    addEventListener('pointercancel', up);
  });


  // --- ADAPTARSE AL TAMAÑO DE LA PANTALLA ------------------------------------
  // Si giras el celular o cambias el tamaño de la ventana, se redibuja todo
  // para que la gráfica no se vea cortada. "raf" evita redibujar demasiadas
  // veces seguidas (solo una vez por cuadro de pantalla).
  let raf;
  new ResizeObserver(() => {
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(all);
  }).observe(root);
}
