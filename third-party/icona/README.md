# Icona local

Origen: [Icona.works](https://icona.works/) y
[repositorio oficial fijado](https://github.com/komalverma22/Icona/tree/dd9f5217720a3f459a1f5464125701eb0d51c3a2).
Revisión **dd9f5217720a3f459a1f5464125701eb0d51c3a2**, importada el26/10/2026.

`assets.json` contiene las exportaciones oficiales de **66 variantes SVG** y
**nueve escenas que comparten Jelly de Tasks**, para **85 claves lógicas**. No contiene el
sitio, fuentes, imágenes de marca, React ni el catálogo completo.
`upstream-package.json` es una copia sin modificaciones de la declaración MIT.
Ver [NOTICE](NOTICE.txt) y [términos MIT](LICENSE.txt): upstream declara MIT pero
no entrega un archivo LICENSE separado en esta revisión.

## Reproducción

El build normal sólo usa archivos versionados locales:

```sh
npm run icons:generate
npm run icons:check
```

Para auditar/reimportar desde upstream, usar Node22.22 o posterior y un checkout
local limpio en la revisión indicada. Obtener ese checkout es una acción explícita
de desarrollo, no parte de build ni de la aplicación:

```sh
git clone https://github.com/komalverma22/Icona.git /tmp/icona-source
git -C /tmp/icona-source checkout dd9f5217720a3f459a1f5464125701eb0d51c3a2
node scripts/import-icona.mjs /tmp/icona-source
npm run icons:generate
npm run icons:check
```

El importador comprueba revisión y limpieza y usa `staticSvg`/`generateCode` del
exportador oficial. SHA256 de `assets.json` en esta importación:
`31329e6519c9aad5cb114cff271a7c4041d4ef4583e94ffb2ff700e4f3aa09e1`.
El generador valida tags/atributos/referencias y aísla máscaras por instancia.
Fill usa stroke1.5 oficial; las variantes sin Fill se exportan bold2.4.

## Equivalencias y adaptaciones

La correspondencia completa vive en [icon-catalog.mjs](../../scripts/icon-catalog.mjs).
Se conservan las claves de registros, sin reescribir entidades:

| Clave o concepto anterior | Icona elegido |
| --- | --- |
| contact / circle-user / person | User Fill |
| user-plus / users | Composición de User Fill con Plus / otro User Fill |
| phone-call | Headphones Fill, atención por audio |
| shapes / collections | Layers Fill |
| forward | Dos Arrow Right bold |
| columns / app-window | Split View Fill |
| trophy / Goals | Flag Fill, meta |
| list | Menu bold |
| pen | Pen Tool Fill |
| asterisk | Sparkles Fill |
| file / file-text / notebook | File Fill, con líneas recortadas |
| task / list-checks | Checklist bold |
| inbox / archive | Inbox Zero Fill |
| circle | Loader bold |
| book | Bookmark Fill, lectura |
| audio | Volume bold |
| globe / Web | Compass Fill, navegación |
| plane / Travel | Paper Plane Fill |
| briefcase / Work | Kanban Fill |
| graduation / Study | Lightbulb Fill |
| dumbbell | Zap Fill, energía |
| gamepad | Dice Fill, juegos |
| wrench | Settings bold |
| menu / sidebar-right | Sidebar Collapse Fill / reflejado |
| calendar-week | Calendar Fill |
| calendar-clock | Snooze Fill, fecha diferida |
| circle-dot | Focus Ring Fill |
| hammer | Edit bold |
| sliders | Filter Fill |
| expand | Resize Handle bold |
| arrow-up-right | Arrow bold diagonal |
| save | Download bold, guardar |
| ellipsis | More bold |
| lock-open | Lock Fill con arco girado sobre su bisagra |

Las otras claves mantienen el concepto homónimo de Icona. Las adaptaciones
transforman/componen geometría oficial; no fabrican relleno sobre paths outline.

## Movimiento

Por elección del usuario, los nueve accesos laterales usan el mismo Jelly que
Tasks. El importador exporta la receta oficial checklist/Jelly una vez y la aplica
a un grupo .fx que contiene cada SVG estático. Los nueve comparten los mismos
keyframes e intensidad. El ajuste posterior del usuario ralentiza el ciclo de700
a1000ms sin cambiar la curva o las figuras. La duración se fija una sola vez en
`SIDEBAR_MOTION_DURATION_MS` dentro de `scripts/icon-catalog.mjs`.
El generador comparte una única definición CSS, sin depender de que el catálogo
upstream ofrezca Jelly entre las variantes de cada figura. A los1050ms restaura
los nodos estáticos y retira estilos. CSS, IDs y keyframes quedan aislados por
reproducción. Sin partículas, hover ni bucles permanentes.
