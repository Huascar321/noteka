# Anchos del menú y Side peek

El menú izquierdo se ajusta desde su borde derecho; Side peek, desde su borde
izquierdo. Los máximos son **35% y 65% del ancho de la ventana de Noteka**,
respectivamente. El mínimo deseado es 180/280px y cede ante el máximo en ventanas
estrechas. Las flechas mueven el borde 10px (Shift: 50px, Alt: 1px); Home/End
usan mínimo/máximo. Escape durante el arrastre cancela el ajuste.

El borde del menú queda dentro de su contorno y separado del cuerpo scrolleable:
no aparece scrollbar horizontal, y el scroll vertical sigue permitiendo llegar
a Settings en ventanas bajas. El borde permanece visible al desplazar el contenido.

La preferencia global del dispositivo se guarda en `noteka.panel-widths.v1`
como píxeles preferidos, al soltar el borde o confirmar un ajuste por teclado.
Cerrar/reabrir o reiniciar recupera ese ancho, acotado al viewport actual.
Achicar la ventana no sobrescribe la preferencia: ampliar vuelve a permitirla.
Valores inválidos usan 250px de menú y 45% de Side peek; errores de escritura
conservan el ajuste de sesión y muestran un aviso recuperable.

Un Side peek de **más del 45%** cierra el menú. Volver a abrir el menú oculta ese
peek sin descartar el editor. Elegir el mismo detalle lo revela con su borrador
e historial; elegir otro aplica la guardia habitual antes de reemplazarlo. A
45% exacto pueden coexistir. Achicar Side peek no abre el menú automáticamente.

La expansión/contracción usa 260ms y el easing del menú. Movimiento reducido y
arrastre aplican el tamaño directamente. Ocultar es presentación de sesión;
cerrar explícitamente sigue protegiendo los cambios y también se consideran los
peeks ocultos al abandonar pestaña, workspace o ventana. No cambia SQLite.
