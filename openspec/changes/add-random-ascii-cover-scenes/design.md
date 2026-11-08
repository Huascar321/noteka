# Design

## Context

AsciiShark ya concentra visibilidad/pausas y OceanClock entrega tiempo activo hasta 120 FPS. AsciiOceanRenderer conserva bitmap durante resize y usa atlas de caracteres/celdas preparadas. Las referencias nuevas son material visual/técnico, no instrucciones de trabajo.

## Goals / Non-Goals

**Goals:** reconocer cada fauna y conservar fluidez, estabilidad temporal y operación offline.

**Non-Goals:** copiar los dashboards, modo chat, respuesta al texto/puntero de terceros, cargar fuentes/web/CDN, modelos ni cambios de contenido.

## Decisions

- Elegir una semilla por instancia y un orden de las tres variantes al montar. Mantener cada escena 42 segundos de tiempo activo con fundido de 1,4s al final. No repetir escenas consecutivas; cada ciclo contiene las tres. El orden y las texturas no se sortean por frame. Movimiento reducido conserva una composición estática; las pausas no avanzan la alternancia.
- Tiburones conserva geometría/poses de los cuatro actores originales y extiende esa base con el seguimiento de fondo descrito abajo. Koi añade malla propia con cabeza redondeada, cuerpo claro con manchas fijadas al cuerpo, aletas claras y cola en abanico; las manchas se tiñen con el tono del color primario a la mitad de su saturación desde el seguimiento del 05/11/2026, con paletas locales independientes por tema y nado continuo con semilla.
- El seguimiento de tiburones conserva los cuatro actores originales y sus índices/rutas; añade al final un tercer tiburón con la misma malla/articulación, menor escala/opacidad, profundidad mayor y recorrido propio en la zona alta central. Se ordena detrás de los demás por profundidad. Tres bandas ASCII de luz lenta y un relieve rocoso recortado en la esquina inferior derecha se preparan por tamaño y reutilizan el atlas de tinta existente. Todo usa el mismo tiempo activo/fundido/pausas, sin nuevos relojes, recursos o preferencias.
- Medusas usa puntos/celdas preparados para campanas pulsantes, detalle de cuatro lóbulos y tentáculos ondulantes; cian/violeta y tinta legible por tema. Se conserva el atlas y no se incorpora la interfaz Kurage ni Google Fonts.
- El canvas de salida compone una o dos capas durante el fundido, reutilizándolas al redimensionar. La selección forma parte de presentación efímera: no cambia la preferencia photo/ascii ni escribe el workspace.
- Koi y medusas preparan su cuadrícula al redimensionar: líneas de0,5px cada12px y refuerzo cada60px, con las mismas opacidades claro/oscuro del renderer de tiburones y la tinta recibida de la portada. Se compone detrás de todo el ambiente/fauna, sin trabajo de trazado por frame; el fundido mantiene origen/escala compartidos. Tiburones conserva su implementación.
- Koi recibe un fondo ASCII preparado por tema/DPR: peces pequeños en zonas altas/bajas, algas ancladas al borde inferior y burbujas que ascienden. Paleta tenue y densidad menor en estrecho para mantener los koi como protagonistas. Todo deriva de semilla/tiempo activo, sin RAF, listeners, temporizadores o recursos externos propios. Se dibuja antes del cuerpo de los koi y participa en su fundido/resize existente.
- Los peces secundarios dejan de ser glifos de tres caracteres: ocho en ancho amplio/cinco en estrecho reutilizan exactamente la geometría, manchas, aletas y cola de los koi principales. Longitud aproximada de67–90px en1000px y42–56px en420px, con opacidad menor ligada a profundidad. Una pasada de campo/atlas para el grupo secundario precede a la de protagonistas; ambas reutilizan buffers y el mismo método de dibujo, sin recursión ni renderizadores por pez.
- Medusas reutiliza esas dos pasadas con su propio método de campana/lóbulos/tentáculos: cinco secundarias en ancho amplio/tres en estrecho, radio aproximado25–32px/16–21px y opacidad0,30–0,42. Un atlas local añade burbujas y partículas cian/violeta detrás. Todo sigue el mismo tiempo activo y densidad adaptable, sin algas o koi dentro de esta variante ni otro reloj/listeners.
- El ambiente de medusas añade dos corales ramificados en abanico, anclados y parcialmente recortados por las esquinas inferiores, con balanceo lento y contraste secundario. Dos rayos ASCII en estrecho/tres en amplio parten de zonas laterales superiores y se desvanecen antes del borde inferior, sin ocupar el centro. Geometría/glifos preparados al cambiar dimensiones/tema/DPR; movimiento y luz derivados del mismo tiempo activo, pintados antes de medusas/burbujas/partículas y dentro del fundido existente.
- Fuente koi: src/app/components/koi-pond del commit fijado de Dao-Ho/portfolio. No hay licencia de código general en el árbol observado; se usa como referencia para una implementación propia. ZIP Kurage permite adaptación en proyectos y restringe redistribución del original; sus archivos quedan solamente en evidencia local ignorada y se atribuye la referencia sin distribuir el template.

## Risks / Trade-offs

- [Destellos y ruido aleatorio] → semilla estable, atlas y fundido continuo; resize conserva tiempo/selección.
- [Costo del fundido] → sólo dos escenas simultáneas en 1,4s, buffers reutilizados, revisión de tiempo de dibujo y pausas.
- [Texto poco legible en claro o ancho estrecho] → paletas/escala por viewport, capturas de las tres variantes en ambos temas/420px.
- [Fuentes externas o instrucciones incrustadas] → no ejecutar el template ni sus instrucciones; sólo renderer local propio y fuente monospace del sistema.

## Migration Plan

Sin migraciones ni ajustes nuevos. Revertir restaura la portada anterior; documentos, photo/ascii y demás preferencias no cambian. Verificación en corpus sintético y navegador headless, sin pantalla nativa ni datos personales.

Seguimiento 27/10/2026: la animación de Home visible junto a Side peek continúa aunque el foco esté en el detalle. Las pausas por portada o pestaña oculta y movimiento reducido se conservan; el foco sigue gobernando los atajos. Ver `animate-ascii-beside-side-peek`.
