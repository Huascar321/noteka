# Verificación de colores de Projects Board

29/10/2026. Se observaron en la ventana nativa oscura las superficies neutras,
las etiquetas distinguibles y los acentos atenuados. La compilación pasó con cero
diagnósticos Svelte (se mantiene el aviso de tamaño de chunk). En WebKit headless
con tema claro, el recorrido sintético de Board pasó 17 comprobaciones, incluido
el destino del arrastre y la persistencia. La revisión visual nativa del tema
claro no se realizó; no se cambió la preferencia de tema del usuario.

Seguimiento: se añadió un matiz de estado del 8 % a cada superficie y se
restauraron los colores de los badges de Projects. El resultado se inspeccionó
en la ventana nativa oscura; build y validación OpenSpec aprobaron. El tema
claro usa las mismas variables adaptadas al tema, sin nueva revisión visual.
