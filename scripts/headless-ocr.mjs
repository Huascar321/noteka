// Sólo pruebas: OCR real sobre una imagen sintética y control de errores headless.
import { createHash, randomUUID } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, readdirSync, copyFileSync, realpathSync } from 'node:fs';
import { join } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
const hash = x => createHash('sha256').update(x).digest('hex');
export function createOcrControls(root, confined, cli) {
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
        ? { api_version: 1, request_id: randomUUID(), ok: false, error: { code: 'storage_error', message: '<script>OCR lector fallo sintético</script>', details: { reason: 'headless_ocr_injected' } } }
        : await cli(command.operation, command.input, ws);
      if (['extraction.get', 'extraction.list', 'extraction.ocr_regions', 'extraction.search', 'entity.get', 'entity.list', 'search.query'].includes(command.operation) || command.operation === 'extraction.control' && command.input.action === 'get' || command.operation === 'extraction.index' && command.input.action === 'get') {
        entry.readonly_unchanged = JSON.stringify(before) === JSON.stringify(snapshot(ws)); if (!entry.readonly_unchanged) throw new Error('Una lectura OCR escribió');
      }
      writeFileSync(join(root, 'ocr-operations.json'), JSON.stringify(operations, null, 2)); return result;
    },
    async control(step, input) {
      if (step === 'ocr_seed') {
        if (!['dark', 'light'].includes(input.theme)) throw new Error('Tema inválido');
        const source = realpathSync(process.env.NOTEKA_OCR_UI_SOURCE), allowed = realpathSync(process.env.NOTEKA_TEST_TMP);
        if (!source.startsWith(allowed + '/')) throw new Error('Fuente fuera de evidencia externa');
        const base = confined(join(root, `ocr-${input.theme}-${randomUUID()}`)); mkdirSync(base);
        const ws = join(base, 'workspace'), image = join(base, 'regions.png'), bad = join(base, 'invalid.png'); copyFileSync(source, image); writeFileSync(bad, 'invalid synthetic image');
        await data(ws, 'workspace.create', { timezone: 'UTC' });
        const attachment = await data(ws, 'attachment.import', { source_path: image }), invalid = await data(ws, 'attachment.import', { source_path: bad });
        const note = await data(ws, 'entity.create', { kind: 'note', title: 'OCR headless sintético', document: { schema_version: 2, blocks: [{ id: 'ocr-note', type: 'paragraph', content: [{ type: 'text', text: 'notekaocr original del buffer' }] }] } });
        const f = { workspace: ws, note, attachment, invalid }; fixtures.set(ws, f); return f;
      }
      const f = known(input.workspace);
      if (step === 'ocr_snapshot') return snapshot(f.workspace);
      if (step === 'ocr_operations') return operations.filter(o => o.workspace === f.workspace);
      if (step === 'ocr_instrument') { instruments.set(f.workspace, input.instrument); return true; }
      if (step === 'ocr_cli') {
        if (!['extraction.ocr_regions', 'extraction.get', 'extraction.search', 'extraction.index', 'extraction.retry'].includes(input.operation)) throw new Error('Operación OCR de prueba no permitida');
        return cli(input.operation, input.input, f.workspace);
      }
      if (step === 'ocr_report') {
        if (!['dark', 'light'].includes(input.theme) || !input.ok) throw new Error('Reporte inválido');
        writeFileSync(join(root, `ocr-${input.theme}-results.json`), JSON.stringify(input, null, 2)); return true;
      }
      throw new Error('Control OCR inválido');
    },
  };
}
