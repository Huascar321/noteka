# Spec Delta

## MODIFIED Requirements

### Requirement: Portada local y animación ASCII accesible

Home SHALL ofrecer la portada local recuperada de la referencia autorizada y una alternativa ASCII con tiburones que nadan y giran mediante cambios continuos de forma, orientación y profundidad. MUST funcionar offline en claro/oscuro, conservar la preferencia de portada separada de los datos y detener movimiento cuando se pide movimiento reducido, la portada no es visible, el documento está oculto o la pestaña está inactiva.

La alternativa ASCII SHALL distinguir morro/aletas/cola, coordinar giros y ondulación posterior con impulso/deslizamiento y estabilizar caracteres sin ruido temporal ni estelas de celdas vacías. MUST reutilizar buffers y caracteres preparados, conservar velocidad basada en tiempo y programar dibujo con requestAnimationFrame hasta120 FPS, sujeto a la cadencia/presupuesto reales. Partículas, luz y bancos de peces ASCII SHALL ser ambientales, acotados y locales; sin consultas de datos, recursos remotos ni nuevos ajustes persistentes.

La presentación SHALL seguir la referencia de video autorizada mediante vista cenital, celdas sombreadas con letras contrastantes y acentos deterministas discretos, y grilla fina de fondo. MUST incluir tiburones y peces de cuerpo compacto/alargado con geometrías y cadencias propias, conservando render procedural, estabilidad temporal y las pausas/cadencia existentes. La interfaz del video no se incorpora a Home.

#### Scenario: Foto y ASCII en ambos temas
- **WHEN** se alterna foto/ASCII y tema con edición pendiente en otra pestaña
- **THEN** la portada elegida se muestra con recursos empaquetados y contraste propio del tema, y no se escriben ni descartan entidades ni buffers

#### Scenario: Movimiento reducido y desmontaje
- **WHEN** se activa movimiento reducido, se oculta la portada o se cierra su pestaña
- **THEN** queda una composición estática o se suspende el render, sin un bucle de animación activo para contenido oculto ni listeners retenidos tras desmontar

#### Scenario: Cadencia y reanudación
- **WHEN** el navegador entrega callbacks a distintas frecuencias o la animación se reanuda tras una pausa
- **THEN** se dibuja como máximo a120 FPS, el desplazamiento depende del tiempo activo y no de la cantidad de cuadros, y la pausa no produce saltos de recuperación

#### Scenario: Escena ambiental estable
- **WHEN** se dibuja o redimensiona la portada en cualquiera de los temas
- **THEN** tiburones y detalle ambiental conservan geometría finita y caracteres preparados con contraste del tema, el historial se reinicia al cambiar dimensiones y no se añaden requests ni escrituras del workspace

#### Scenario: Diversidad y textura de la referencia
- **WHEN** se observa la portada en movimiento o como composición estática
- **THEN** se distinguen las siluetas de tiburón, pez compacto y pez alargado, con sus propias aletas/colas y nado, mientras celdas/letras mantienen sombreado y acentos estables sin usar fotogramas como recurso
