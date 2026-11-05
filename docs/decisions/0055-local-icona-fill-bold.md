# ADR0055 — Icona local con Fill y bold

Fecha:26/10/2026. Estado: aceptada. Sustituye catálogo y reproductor de ADR0041;
conserva su política de movimiento explícito y las imágenes/campos de ADR0034/0036.

Actualización26/10: el usuario eligió el Jelly de Tasks para los nueve accesos
laterales. `unify-sidebar-jelly-motion` conserva esa exportación y aplica sus
keyframes al grupo de cada figura, compartiendo CSS. El resto de esta ADR
conserva el contexto y las medidas de la entrega inicial;
ver [validación posterior](../verification-sidebar-jelly.md).
El pedido posterior de menor velocidad fija el ciclo en1000ms y limpieza1050ms,
sin cambiar la curva ni intensidad, con duración centralizada en el catálogo.

## Context

El usuario pide reemplazar todos los iconos del set por Icona.works para obtener
relleno/bold y conservar animaciones. La UI ya centraliza85 nombres en AppIcon y
nueve botones laterales con carga diferida. Icona exporta SVG/CSS y declara MIT en
su sitio y package metadata; el paquete React no es necesario para Svelte.

## Decision

Fijar revisión dd9f5217720a3f459a1f5464125701eb0d51c3a2 y guardar66 variantes
exportadas más nueve escenas CSS. Generar sólo71 árboles estáticos deduplicados
para las85 claves, con Fill1.5 oficial u outline bold2.4. Conservar claves, tamaños,
color, etiquetas, fallback e imágenes. Las figuras inexistentes usan equivalentes
documentados y cinco adaptaciones de geometría oficial. No convertir paths de
outline en relleno artificial ni cambiar datos para renombrar iconos.

El importador es explícito y exige checkout limpio/revisión exacta. El build usa
únicamente assets versionados y valida tags/atributos/referencias locales; máscaras
únicas por instancia. Se distribuyen declaración/licencia/procedencia MIT. Upstream
no incluye LICENSE separado: NOTICE distingue la declaración original del texto
estándar reproducido con atribución, sin presentarlo como copia inexistente.

Sólo el clic lateral importa escenas CSS oficiales. El reproductor monta una
escena y estilos aislados por ejecución, hace un ciclo de700ms y restaura a750ms
los mismos nodos originales. Mantiene cancelación global, clic repetido, desmontaje,
visibility y reduced motion, incluso durante la importación. Sin hover, canvas,
partículas o bucles recurrentes. AppIcon sigue siendo estático y no importa motions.

## Alternatives and consequences

Usar componentes React añadiría otra UI/runtime. Depender de registry/CDN rompería
el arranque offline. Aplicar fill a todos los outlines cerraría detalles y dibujaría
mal flechas/controles. Las variantes seleccionadas mantienen recortes y trazos
propios del proveedor; los equivalentes pueden cambiar la metáfora visual anterior.

Se retira @animateicons/vanilla y su excepción del prebundler. La escena diferida
actual mide30,32kB/4,51kB gzip; el build principal conserva su aviso de tamaño, sin
cambiar umbrales ni afirmar una optimización global. Marca/controles nativos y
marcas de estado de tareas conservados. Sin React, nuevas dependencias, Rust,
migración, credenciales, proveedor remoto ni escrituras del catálogo a SQLite.

Ver [verificación](../verification-icona.md). Chromium/WebKit headless no acreditan
IPC, cierre de ventana ni aceptación visual nativa de Tauri.
