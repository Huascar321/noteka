# Portada de océano ASCII

Desde el seguimiento del 26/10/2026, esta composición es una de las tres variantes
de la portada. Alterna con koi naranja/blanco y medusas cian/violeta, con inicio y
orden aleatorios estables. Ver [escenas, ritmo y referencias](ascii-cover-scenes.md)
y [verificación](verification-ascii-cover-scenes.md). La geometría y renderer de
tiburones descritos abajo permanecen intactos.

Home conserva ASCII/Foto y su preferencia local anterior. La escena procedural
combina dos tiburones, un pez de cuerpo compacto y otro alargado. Los tipos de
pez son categorías visuales, sin atribuir una especie biológica al video de
referencia. Cada tipo tiene su propia malla, aletas, cola y ritmo de ondulación.
La cámara es cenital. Un esqueleto de ochenta segmentos de longitud constante
distribuye el giro a lo largo del cuerpo, con ondulación creciente hacia la cola.
Las secciones, sus normales y las aletas giran juntas: el cuerpo forma una curva
sin alargar el posterior y el sombreado acompaña esa postura. La cabeza conserva
su dirección durante cada ciclo de cola. La aleta caudal está ligeramente
inclinada para distinguir sus dos lóbulos desde arriba.

Los recorridos reducen la velocidad al entrar en los extremos y combinan
impulso/deslizamiento con ritmos propios. La curva corporal sigue la curvatura
del recorrido; se limita para evitar pliegues cerrados. Escala/opacidad dependen
de la profundidad decorativa y el orden de dibujo sigue esa profundidad.

Los centros de los tiburones están repartidos a izquierda/derecha, con recorridos
que abarcan cada uno el 32 % del ancho. Los peces pequeños exploran el 34 % del
ancho en zonas altas/bajas y con fases diferentes. Así conservan posiciones
variadas y cruces ocasionales sin acumular casi todo el elenco en la misma zona.
La distribución es continua y determinista, sin teletransportes ni un simulador
de colisiones. Ver [revisión de dispersión](verification-ocean-spread.md).

La textura sigue los videos de día y noche aportados por el usuario: celdas de
6×8 px, letras contrastantes, tiburones gris azulados y peces oliva/dorados.
Cada tema tiene paletas propias, con acentos pastel discretos. Los acentos y la variación de glifos son
deterministas, ligados a la grilla local del animal. No se usa ruido por cuadro.

La grilla viaja con cada animal para permitir una traslación continua. Un atlas
local prepara las celdas y glifos por tema/DPR; geometría, profundidad y sombreado
reutilizan buffers. Suavizado e histéresis de tonos estabilizan los caracteres,
y las celdas vacías se limpian sin estela. El fondo usa una grilla fina, partículas
y tres bancos lejanos de puntos y pequeños trazos; los rayos de la iteración anterior se
retiraron para seguir la referencia.

No se incorporan fotogramas ni el video al producto. Todo es decorativo y local,
sin imágenes descargadas, consultas del workspace ni funciones de IA. Claro y
oscuro conservan sus paletas; la resolución interna se limita a DPR 2.

## Cadencia y pausas

La animación se programa con requestAnimationFrame **hasta 120 FPS**. El navegador,
la pantalla y el presupuesto de dibujo determinan los FPS visibles. El reloj usa
timestamps y conserva la velocidad en distintas frecuencias.

Al ocultar la portada, cambiar a una pestaña inactiva u ocultar el documento se
suspende el reloj y se cancela RAF. Reanudar no recupera el tiempo pausado.
Movimiento reducido muestra una composición estática a tiempo 0 para toda la
escena. Cambiar tamaño reconstruye buffers y repinta la escena actual antes de
que el navegador presente el cuadro, sin suspender el reloj activo. El atlas se
reutiliza mientras no cambien DPR/tema/color. Desmontar retira los
observadores/listeners y cancela el cuadro pendiente.
Ver [corrección del parpadeo al redimensionar](verification-ascii-resize-flicker.md).

No se añaden controles, preferencias, dependencias ni formatos persistentes.
Home, datos y la portada Foto conservan su comportamiento anterior.
Ver [revisión de día/noche y nado articulado](verification-shark-swimming.md),
[referencia diurna anterior](verification-shark-day-reference.md) y
[mediciones de la iteración anterior](verification-ascii-ocean-cover.md).
