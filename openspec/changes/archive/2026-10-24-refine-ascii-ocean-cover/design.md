# Design

## Context

AsciiShark ya usa Canvas2D, tres poses, superficie de puntos, z-buffer y observadores de tamaño/intersección/tema/visibilidad. Su máximo era24 FPS, con trigonometría por punto, tres buffers nuevos por cuadro y ruido ligado a celdas. HomePanel y la preferencia ASCII/Foto no necesitan cambios. El usuario autorizó hasta120 FPS y detalle adicional; no pidió interacción ni modelos externos.

## Goals / Non-Goals

Mejorar silueta/nado/estabilidad y añadir ambiente discreto conservando todas las pausas. Sin modificar Home, datos, contratos, editores o preferencias. No prometer120 FPS físicos desde un navegador limitado a60Hz ni desde una simulación de reloj.

## Decisions

1. Malla propia con perfil corporal definido, morro y pedúnculo separados, aletas barridas y cola asimétrica; detalles de ojos/branquias como glifos. Conservar las funciones geométricas actuales y preparar coordenadas/normales/deformación para proyección por pose sin objetos por punto. Movimiento continuo con variación de impulso, ondulación creciente hacia cola e inclinación asociada al giro.
2. Renderer Canvas2D separado y sin dependencias. Buffers reutilizados por tiburón, atlas local de glifos según DPR/tema y sombreado con histéresis temporal. La grilla viaja con cada animal: la traslación se compone de forma continua y los caracteres no saltan por atravesar la grilla de pantalla. Se elimina ruido aleatorio y se limpia de inmediato una celda vacía; no usar estelas para disimular flicker. Rehacer cachés sólo al cambiar dimensiones/DPR/tema.
3. Reloj con timestamps RAF, límite120 y acumulación de tiempo activo independiente del render. Cadencias30/60/90/120/144/240 se comprueban con timestamps controlados; no son prueba de una pantalla120Hz. Los observadores existentes cancelan RAF y suspenden el reloj; movimiento reducido dibuja una composición estática a tiempo0. Respetar limpieza al desmontar y retorno de pestaña sin salto.
4. Ambiente acotado: luz tenue preparada, partículas deterministas y bancos pequeños de peces ASCII con menor contraste. Sin añadir nuevas fuentes de datos, archivos de imagen, fetch o ajustes de producto. Mantener sujetos principales legibles en ventanas estrechas y ambos temas.

## Risks / Trade-offs

- El coste de Canvas/compositor y la frecuencia de pantalla limitan el FPS visible. Medir coste de render y cadencia real por separado; conservar resultados con tamaño/DPR/entorno.
- Una grilla de caracteres limita el detalle espacial aunque la posición se actualice a120. Histéresis y contraste buscan reducir saltos sin difuminar siluetas.
- El ambiente añade coste y puede distraer; mantener cantidad/contraste bajos y revisarlo visualmente.

## Migration Plan

Sin formatos persistentes ni migración. Baseline privado de41 archivos antes de editar; no abrir ni copiar el workspace personal para una portada. Mantener bytes de HomePanel/preferencia, otros detalles, Rust/contratos/locks, dieciséis specs y Meeting Notes0/19. Evidencia de compilación/geometría/reloj/render y capturas fuera de Git. Revertir presentación no requiere restaurar datos.
