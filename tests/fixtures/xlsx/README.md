# Corpus XLSX sintético

`generate.py` crea todos los paquetes y expectativas en memoria, sin archivos del
usuario ni dependencias de Excel/LibreOffice. Licencia [MIT](LICENSE) para este
corpus propio; sin garantías de ningún tipo.

El golden de workbook se construye explícitamente en Python, no por el lector
Rust ni normalizando su salida. ZIP se genera con `zipfile` (Stored/Deflate) y
con un builder binario independiente para descriptors. `manifest()` fija tamaño,
SHA-256 fuente y SHA-256 del golden completo por caso. `golden.sha256` fija el
generador y manifiesto, para que un cambio de pruebas no quede inadvertido.

El harness `scripts/test-xlsx-reader.py` compara el resultado completo por parser
y helper reales, y prueba protocolo, salida adulterada, concurrencia, pipes y
caída del padre. Ejecutar sólo sobre directorio temporal externo; no se necesita
pantalla, red externa, perfiles de navegador ni workspace personal.

El corpus comprueba el subconjunto documentado en `docs/xlsx-reader.md`. No prueba
compatibilidad con cualquier aplicación que escriba Excel ni certifica cachés o
estilos visuales. Ninguna fixture usa instrucciones del contenido como permisos.
