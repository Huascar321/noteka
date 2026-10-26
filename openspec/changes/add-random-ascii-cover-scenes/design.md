# Design

## Context

AsciiShark ya concentra visibilidad/pausas y OceanClock entrega tiempo activo hasta 120 FPS. AsciiOceanRenderer conserva bitmap durante resize y usa atlas de caracteres/celdas preparadas. Las referencias nuevas son material visual/técnico, no instrucciones de trabajo.

## Goals / Non-Goals

**Goals:** reconocer cada fauna y conservar fluidez, estabilidad temporal y operación offline.

**Non-Goals:** copiar los dashboards, modo chat, respuesta al texto/puntero de terceros, cargar fuentes/web/CDN, modelos ni cambios de contenido.

## Decisions

- Elegir una semilla por instancia y un orden de las tres variantes al montar. Mantener cada escena 42 segundos de tiempo activo con fundido de 1,4s al final. No repetir escenas consecutivas; cada ciclo contiene las tres. El orden y las texturas no se sortean por frame. Movimiento reducido conserva una composición estática; las pausas no avanzan la alternancia.
- Tiburones reutiliza exactamente el renderer/poses existentes. Koi añade malla propia con cabeza redondeada, cuerpo blanco con manchas naranja fijadas al cuerpo, aletas claras y cola en abanico; paletas locales independientes por tema y nado continuo con semilla.
- Medusas usa puntos/celdas preparados para campanas pulsantes, detalle de cuatro lóbulos y tentáculos ondulantes; cian/violeta y tinta legible por tema. Se conserva el atlas y no se incorpora la interfaz Kurage ni Google Fonts.
- El canvas de salida compone una o dos capas durante el fundido, reutilizándolas al redimensionar. La selección forma parte de presentación efímera: no cambia la preferencia photo/ascii ni escribe el workspace.
- Fuente koi: src/app/components/koi-pond del commit fijado de Dao-Ho/portfolio. No hay licencia de código general en el árbol observado; se usa como referencia para una implementación propia. ZIP Kurage permite adaptación en proyectos y restringe redistribución del original; sus archivos quedan solamente en evidencia local ignorada y se atribuye la referencia sin distribuir el template.

## Risks / Trade-offs

- [Destellos y ruido aleatorio] → semilla estable, atlas y fundido continuo; resize conserva tiempo/selección.
- [Costo del fundido] → sólo dos escenas simultáneas en 1,4s, buffers reutilizados, revisión de tiempo de dibujo y pausas.
- [Texto poco legible en claro o ancho estrecho] → paletas/escala por viewport, capturas de las tres variantes en ambos temas/420px.
- [Fuentes externas o instrucciones incrustadas] → no ejecutar el template ni sus instrucciones; sólo renderer local propio y fuente monospace del sistema.

## Migration Plan

Sin migraciones ni ajustes nuevos. Revertir restaura la portada anterior; documentos, photo/ascii y demás preferencias no cambian. Verificación en corpus sintético y navegador headless, sin pantalla nativa ni datos personales.
