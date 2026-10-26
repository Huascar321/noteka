// Sólo pruebas: OCR real sobre una imagen sintética y control de errores headless.
import { createHash, randomUUID } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, readdirSync, copyFileSync, realpathSync } from 'node:fs';
import { join } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
const hash = x => createHash('sha256').update(x).digest('hex');
export function createPdfOcrControls(root, confined, cli) {
  const fixtures = new Map(), operations = [], instruments = new Map();
  const data = async (ws, operation, input) => { const r = await cli(operation, input, ws); if (!r.ok) throw new Error(JSON.stringify(r)); return r.data; };
  const known = ws => { confined(ws); if (!fixtures.has(ws)) throw new Error('Fixture OCR desconocida'); return fixtures.get(ws); };
  function snapshot(ws) {
    known(ws); const db = new DatabaseSync(join(ws, 'noteka.sqlite'), { readOnly: true });
    try {
      const tables = Object.fromEntries(db.prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name").all().map(({ name }) => [name, db.prepare('SELECT * FROM "' + name.replaceAll('"', '""') + '"').all().map(r => JSON.stringify(r)).sort()]));
      const files = Object.fromEntries(readdirSync(join(ws, 'attachments')).map(name => [name, hash(readFileSync(join(ws, 'attachments', name)))]));
      return { logical: hash(JSON.stringify(tables)), files };
    } finally { db.close(); }
  }
  return {
    async run(ws, command, fallback) {
      if (!fixtures.has(ws)) return fallback();
      const entry = { operation: command.operation, input: command.input, workspace: ws }; operations.push(entry);
      const before = snapshot(ws), instrument = instruments.get(ws);
      if (instrument?.operation === command.operation && instrument.delay) await new Promise(r => setTimeout(r, instrument.delay));
      const result = instrument?.operation === command.operation && instrument.failure
        ? { api_version: 1, request_id: randomUUID(), ok: false, error: { code: 'storage_error', message: '<script>OCR lector fallo sintético</script>', details: { reason: 'headless_pdfocr_injected' } } }
        : await cli(command.operation, command.input, ws);
      // Fixture de transporte para verificar el escape; no representa salida del OCR.
      if (command.operation === 'extraction.pdf_ocr_table' && instrument?.operation === command.operation && instrument.hostileText && result.ok) {
        const region = result.data.items.flatMap(cell => cell.regions)[0];
        if (!region) throw new Error('Fixture de tabla sin región');
        region.text = '<img src=x onerror="window.__tableExecuted=true"><script>window.__tableExecuted=true</script>';
        entry.synthetic_transport_text = true;
      }
      if (['extraction.get', 'extraction.list', 'extraction.pdf_ocr_regions', 'extraction.pdf_ocr_table', 'extraction.search', 'entity.get', 'entity.list', 'search.query'].includes(command.operation) || command.operation === 'extraction.control' && command.input.action === 'get' || command.operation === 'extraction.index' && command.input.action === 'get') {
        entry.readonly_unchanged = JSON.stringify(before) === JSON.stringify(snapshot(ws)); if (!entry.readonly_unchanged) throw new Error('Una lectura OCR escribió');
      }
      writeFileSync(join(root, 'pdfocr-operations.json'), JSON.stringify(operations, null, 2)); return result;
    },
    async control(step, input) {
      if (step === 'pdfocr_seed' || step === 'pdftable_seed') {
        if (!['dark', 'light'].includes(input.theme)) throw new Error('Tema inválido');
        const source = realpathSync(step === 'pdftable_seed' ? process.env.NOTEKA_PDF_TABLE_UI_SOURCE : process.env.NOTEKA_PDF_OCR_UI_SOURCE), allowed = realpathSync(process.env.NOTEKA_TEST_TMP);
        if (!source.startsWith(allowed + '/')) throw new Error('Fuente fuera de evidencia externa');
        const base = confined(join(root, `pdfocr-${input.theme}-${randomUUID()}`)); mkdirSync(base);
        const ws = join(base, 'workspace'), image = join(base, 'regions.pdf'), bad = join(base, 'invalid.pdf'); copyFileSync(source, image); writeFileSync(bad, 'invalid synthetic image');
        await data(ws, 'workspace.create', { timezone: 'UTC' });
        const attachment = await data(ws, 'attachment.import', { source_path: image }), invalid = await data(ws, 'attachment.import', { source_path: bad });
        const note = await data(ws, 'entity.create', { kind: 'note', title: 'PDF OCR headless sintético', document: { schema_version: 2, blocks: [{ id: 'ocr-note', type: 'paragraph', content: [{ type: 'text', text: step === 'pdftable_seed' ? 'noteka original del buffer Alpha' : 'noteka original del buffer' }] }] } });
        const f = { workspace: ws, note, attachment, invalid }; fixtures.set(ws, f); return f;
      }
      const f = known(input.workspace);
      if (step === 'pdfocr_snapshot') return snapshot(f.workspace);
      if (step === 'pdfocr_operations') return operations.filter(o => o.workspace === f.workspace);
      if (step === 'pdfocr_instrument') { instruments.set(f.workspace, input.instrument); return true; }
      if (step === 'pdfocr_cli') {
        if (!['extraction.pdf_ocr_regions', 'extraction.pdf_ocr_table', 'extraction.get', 'extraction.search', 'extraction.index', 'extraction.retry'].includes(input.operation)) throw new Error('Operación OCR de prueba no permitida');
        return cli(input.operation, input.input, f.workspace);
      }
      if (step === 'pdftable_tamper') {
        const target = join(f.workspace, 'attachments', f.attachment.id);
        writeFileSync(target, input.enabled ? Buffer.from('alteración sintética') : readFileSync(join(f.workspace, '..', 'regions.pdf'))); return true;
      }
      if (step === 'pdftable_corrupt') {
        const db = new DatabaseSync(join(f.workspace, 'noteka.sqlite'));
        try {
          if (input.enabled) { const chunk = db.prepare('SELECT generation_id,ordinal,text FROM extraction_pdf_ocr_chunks ORDER BY ordinal DESC LIMIT 1').get(); if (f.savedChunk) throw new Error('Ya alterado'); f.savedChunk = chunk; db.prepare('UPDATE extraction_pdf_ocr_chunks SET text=? WHERE generation_id=? AND ordinal=?').run('x' + chunk.text.slice(1), chunk.generation_id, chunk.ordinal); }
          else { const chunk = f.savedChunk; if (!chunk) throw new Error('Sin alteración'); db.prepare('UPDATE extraction_pdf_ocr_chunks SET text=? WHERE generation_id=? AND ordinal=?').run(chunk.text, chunk.generation_id, chunk.ordinal); f.savedChunk = null; }
        } finally { db.close(); } return true;
      }
      if (step === 'pdftable_report') {
        if (!['dark', 'light'].includes(input.theme) || !input.ok) throw new Error('Reporte inválido');
        writeFileSync(join(root, `pdftable-${input.theme}-results.json`), JSON.stringify(input, null, 2)); return true;
      }
      if (step === 'pdfocr_report') {
        if (!['dark', 'light'].includes(input.theme) || !input.ok) throw new Error('Reporte inválido');
        writeFileSync(join(root, `pdfocr-${input.theme}-results.json`), JSON.stringify(input, null, 2)); return true;
      }
      throw new Error('Control OCR inválido');
    },
  };
}
