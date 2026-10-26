# Detalles de Goal y Person

Los detalles siguen referencias reales de Notion identificadas por su origen.
La presentación usa los valores, relaciones y operaciones actuales de Noteka.
Abrir una página no modifica sus datos ni sincroniza Notion.

## Goal

Status, Tag y Progress se presentan en una fila compacta. Status prepara el
borrador; Tag abre el original y permite acceder al editor de relaciones.
Goal Set, Target Deadline y Achieved siguen editables en View details, junto a
la procedencia y propiedades originales de lectura.

Progress usa el resumen Rust de goals_activity, limitado a la página existente
de cien registros. Un Goal excluido o fuera de esa página muestra No disponible;
un fallo ofrece reintento. Sin hitos se distingue de progreso cero. No se calcula
otro porcentaje desde una tabla parcial ni se completa automáticamente el Goal.

Milestones presenta Name, Target Deadline y Date Completed en tabla. Conserva
el orden de la consulta actual, actualización, opciones, apertura, captura de
hitos vinculados y continuación. La biblioteca de hitos conserva List. Los
proyectos relacionados abren sus originales; el editor completo de relaciones
permanece disponible, aunque los enlaces se distribuyan entre Tag y Goal Projects.

El editor existente de Goal se conserva con la misma configuración, documento,
prefijo y guardias. No se agregan estructuras ni nuevas funciones de cuerpo.
Los bloques importados de plantilla, incluidas etiquetas de vistas y orden,
siguen siendo contenido del documento: no se eliminan ni se convierten en
consultas por su apariencia. La tabla de hitos y Goal Projects son controles
locales independientes de esos textos.

## Person

La foto local existente, con su recorte, sirve como portada. El lector mantiene
sus límites de formato/tamaño y un espacio neutro si no puede leerla. No se
descargan fotos al abrir el detalle. Icono y título conservan sus controles.

View details contiene los campos editables existentes: empresa, etiquetas de
relación, cumpleaños, próximo/último check-in y pipeline. Procedencia y campos
desconocidos se conservan. Las otras propiedades privadas de Notion no se copian.

Notes, Projects y Tasks presentan los originales del contexto local. New note
y New task conservan captura vinculada y sus guardias. Los proyectos abren su
detalle y las relaciones se administran con el editor existente. No se añade una
acción nueva de crear proyectos ni se ocultan acciones anteriores.

Las secciones conservan el cursor común, orden y deduplicación por ID. Durante
una página parcial expresan que la ausencia se refiere a lo cargado; un error
permite reintentar. Las relaciones tipadas/contextuales restantes permanecen
en cabecera. Los datos locales pueden contener menos relaciones que Notion.

Person sigue sin documento/editor de cuerpo. Quick Notes, Gift Ideas y las
vistas remotas de reuniones/notas heredadas de proyectos no se implementan por
copiar la referencia. Meeting Notes permanece propuesto 0/19.

## Herramientas y guardado

Goal conserva Exportar/Recursos externos/Adjuntos por ser una Note tipada;
Person conserva Recursos externos. Se mantienen sus condiciones, paneles
montados, tooltip, teclado y guardias. Editar campos o contenido prepara el
borrador; Guardar o ⌘S/Ctrl+S confirma con las operaciones compartidas. Navegar
con cambios pendientes conserva Guardar/Descartar/Cancelar.

Ver [revisión y límites](verification-goal-person-detail.md),
[Goals](goals.md), [People](people-reference.md) y [herramientas](page-tools.md).
