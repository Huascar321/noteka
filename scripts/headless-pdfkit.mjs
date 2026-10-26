// Sólo pruebas: PDFKit real sobre un PDF sintético y control de errores headless.
import { createHash, randomUUID } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, readdirSync, copyFileSync, realpathSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
const hash = x => createHash('sha256').update(x).digest('hex');
export function createPdfkitControls(root, confined, cli) {
  const fixtures = new Map(), operations = [], instruments = new Map();
  const data = async (ws, operation, input) => { const r = await cli(operation, input, ws); if (!r.ok) throw new Error(JSON.stringify(r)); return r.data; };
  const known = ws => { confined(ws); if (!fixtures.has(ws)) throw new Error('Fixture PDFKit desconocida'); return fixtures.get(ws); };
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
        ? { api_version: 1, request_id: randomUUID(), ok: false, error: { code: 'storage_error', message: '<script>PDFKit lector fallo sintético</script>', details: { reason: 'headless_pdfkit_injected' } } }
        : await cli(command.operation, command.input, ws);
      if (['extraction.get', 'extraction.list', 'extraction.pdfkit_pages', 'extraction.search', 'entity.get', 'entity.list', 'search.query'].includes(command.operation) || command.operation === 'extraction.control' && command.input.action === 'get' || command.operation === 'extraction.index' && command.input.action === 'get') {
        entry.readonly_unchanged = JSON.stringify(before) === JSON.stringify(snapshot(ws)); if (!entry.readonly_unchanged) throw new Error('Una lectura PDFKit escribió');
      }
      writeFileSync(join(root, 'pdfkit-operations.json'), JSON.stringify(operations, null, 2)); return result;
    },
    async control(step, input) {
      if (step === 'pdfkit_seed') {
        if (!['dark', 'light'].includes(input.theme)) throw new Error('Tema inválido');
        const source = realpathSync(process.env.NOTEKA_PDFKIT_UI_SOURCE), allowed = realpathSync(process.env.NOTEKA_TEST_TMP);
        if (!source.startsWith(allowed + '/')) throw new Error('Fuente fuera de evidencia externa');
        const base = confined(join(root, `pdfkit-${input.theme}-${randomUUID()}`)); mkdirSync(base);
        const ws = join(base, 'workspace'), image = join(base, 'pages.pdf'), bad = join(base, 'invalid.pdf'); copyFileSync(source, image); writeFileSync(bad, 'invalid synthetic PDF');
        await data(ws, 'workspace.create', { timezone: 'UTC' });
        const attachment = await data(ws, 'attachment.import', { source_path: image }), invalid = await data(ws, 'attachment.import', { source_path: bad });
        const note = await data(ws, 'entity.create', { kind: 'note', title: 'PDFKit headless sintético', document: { schema_version: 2, blocks: [{ id: 'pdfkit-note', type: 'paragraph', content: [{ type: 'text', text: 'notekapdfkit original del buffer' }] }] } });
        const expected = JSON.parse(readFileSync(join(dirname(source), 'manifest.json'), 'utf8')).pages;
        const f = { workspace: ws, note, attachment, invalid, expected }; fixtures.set(ws, f); return f;
      }
      const f = known(input.workspace);
      if (step === 'pdfkit_snapshot') return snapshot(f.workspace);
      if (step === 'pdfkit_operations') return operations.filter(o => o.workspace === f.workspace);
      if (step === 'pdfkit_instrument') { instruments.set(f.workspace, input.instrument); return true; }
      if (step === 'pdfkit_cli') {
        if (!['extraction.pdfkit_pages', 'extraction.get', 'extraction.search', 'extraction.index', 'extraction.retry'].includes(input.operation)) throw new Error('Operación PDFKit de prueba no permitida');
        return cli(input.operation, input.input, f.workspace);
      }
      if (step === 'pdfkit_report') {
        if (!['dark', 'light'].includes(input.theme) || !input.ok) throw new Error('Reporte inválido');
        writeFileSync(join(root, `pdfkit-${input.theme}-results.json`), JSON.stringify(input, null, 2)); return true;
      }
      throw new Error('Control PDFKit inválido');
    },
  };
}
