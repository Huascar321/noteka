// Sólo aceptación headless, con fuentes sintéticas y CLI real sin red.
import { spawn } from 'node:child_process';
import { createHash, randomUUID } from 'node:crypto';
import { copyFileSync, readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
const hash = value => createHash('sha256').update(value).digest('hex');
export function createPreparationControls(root, confined, cli) {
  const fixtures = new Map(), operations = [], instruments = new Map();
  const data = async (ws, op, input) => { const r = await cli(op, input, ws); if (!r.ok) throw new Error(JSON.stringify(r)); return r.data; };
  const fixture = ws => { confined(ws); const f = fixtures.get(ws); if (!f) throw new Error('Fixture de preparación desconocida'); return f; };
  function snapshot(ws) {
    fixture(ws); const db = new DatabaseSync(confined(join(ws, 'noteka.sqlite')), { readOnly: true });
    try {
      const tables = Object.fromEntries(db.prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name").all().map(({ name }) => [name, db.prepare('SELECT * FROM "' + name.replaceAll('"', '""') + '"').all().map(row => JSON.stringify(row)).sort()]));
      const original = Object.fromEntries(Object.entries(tables).filter(([name]) => ['workspace_meta', 'entities', 'documents', 'relations', 'attachments', 'changes', 'search_fts', 'search_fts_data', 'search_fts_idx', 'search_fts_content', 'search_fts_docsize', 'search_fts_config'].includes(name)));
      const files = Object.fromEntries(readdirSync(confined(join(ws, 'attachments'))).map(name => [name, hash(readFileSync(confined(join(ws, 'attachments', name))))]));
      return { logical: hash(JSON.stringify({ schema: db.prepare('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name').all(), tables })), original: hash(JSON.stringify(original)), files };
    } finally { db.close(); }
  }
  return {
    async run(ws, command, work) {
      if (!fixtures.has(ws)) return work();
      const entry = { operation: command.operation, input: command.input, workspace: ws }; operations.push(entry); writeFileSync(join(root, 'preparation-operations.json'), JSON.stringify(operations, null, 2));
      const instrument = instruments.get(ws);
      if (instrument?.operation === command.operation) {
        if (instrument.delay) await new Promise(r => setTimeout(r, instrument.delay));
        if (instrument.failure) return { api_version: 1, request_id: randomUUID(), ok: false, error: { code: 'storage_error', message: 'Fallo sintético de lectura <script>nunca ejecutar</script>', details: { reason: 'headless_injected' } } };
      }
      const read = ['extraction.list', 'extraction.get'].includes(command.operation) || ['extraction.control', 'extraction.index'].includes(command.operation) && command.input.action === 'get';
      const before = snapshot(ws);
      const result = await work();
      entry.result = result;
      const after = snapshot(ws);
      entry.originals_unchanged = before.original === after.original && JSON.stringify(before.files) === JSON.stringify(after.files);
      if (command.operation.startsWith('extraction.') && !entry.originals_unchanged) throw new Error('Preparación modificó originales o entidades');
      if (read) { entry.readonly_unchanged = before.logical === after.logical; if (!entry.readonly_unchanged) throw new Error('Consulta modificó SQLite'); }
      if (instrument?.operation === command.operation && instrument.transport_after) throw new Error('Transporte perdido después de operación real');
      writeFileSync(join(root, 'preparation-operations.json'), JSON.stringify(operations, null, 2));
      return result;
    },
    async control(step, input) {
      if (step === 'prep_seed') {
        if (!['dark', 'light'].includes(input.theme)) throw new Error('Tema inválido');
        const directory = confined(join(root, `preparation-${input.theme}-${randomUUID()}`));
        await new Promise((done, reject) => {
          const child = spawn('python3', ['-B', resolve('scripts/prepare-evidence-search-fixture.py'), '--root', directory], { env: { ...process.env, NOTEKA_TEST_TMP: root }, stdio: ['ignore', 'pipe', 'pipe'] });
          let out = '', err = ''; child.stdout.on('data', c => out += c); child.stderr.on('data', c => err += c); child.on('error', reject); child.on('close', code => code || err ? reject(new Error(err || out)) : done());
        });
        const f = JSON.parse(readFileSync(join(directory, 'fixture.json'), 'utf8')), ws = f.workspace;
        const sources = {};
        for (const [name, extractor] of [['literal.txt', 'utf8_text'], ['text.pdf', 'pdf_text'], ['cells.xlsx', 'xlsx_cells'], ['empty.xlsx', 'xlsx_cells']]) {
          const source = confined(join(directory, 'fresh-' + name)); copyFileSync(confined(join(directory, name)), source);
          sources[name] = { attachment: await data(ws, 'attachment.import', { source_path: source }), extractor };
        }
        for (const [name, bytes] of [['empty.txt', ''], ['bad.txt', '\u0000invalid']]) {
          const source = confined(join(directory, name)); writeFileSync(source, bytes); sources[name] = { attachment: await data(ws, 'attachment.import', { source_path: source }), extractor: 'utf8_text' };
        }
        const blocks = Object.entries(sources).map(([name, value], i) => ({ id: 'prep-file-' + i, type: 'file', attrs: { attachment_id: value.attachment.id, name: '<script>nunca ejecutar</script> ' + name }, content: [] }));
        blocks.push({ ...blocks[0], id: 'prep-duplicate' });
        const note = await data(ws, 'entity.create', { kind: 'note', title: 'Preparación sintética', document: { schema_version: 2, blocks } });
        const control = await data(ws, 'extraction.control', { action: 'get' }); await data(ws, 'extraction.control', { action: 'set', paused: true, expected_revision: control.revision });
        const queued = f.jobs['queued.txt'];
        // Casos futuros de paginación: no se fabrican generaciones/hits ni éxitos.
        const db = new DatabaseSync(confined(join(ws, 'noteka.sqlite')));
        try {
          const original = db.prepare('SELECT * FROM extraction_jobs WHERE id=?').get(queued.id), columns = Object.keys(original);
          for (let i = 1; i <= 21; i++) {
            const config = JSON.stringify({ chunk_bytes: 4096, version: 100 + i }), digest = createHash('sha256');
            for (const part of ['noteka.extraction.config.v1', config]) { const bytes = Buffer.from(part), size = Buffer.alloc(8); size.writeBigUInt64BE(BigInt(bytes.length)); digest.update(size); digest.update(bytes); }
            const row = { ...original, id: randomUUID(), config, config_hash: digest.digest('hex'), created_at: `2026-10-01T00:00:${String(i).padStart(2, '0')}Z` };
            db.prepare('INSERT INTO extraction_jobs (' + columns.join(',') + ') VALUES (' + columns.map(() => '?').join(',') + ')').run(...columns.map(c => row[c]));
          }
        } finally { db.close(); }
        const value = { ...f, sources, preparation_note: note }; fixtures.set(ws, value); return value;
      }
      fixture(input.workspace);
      switch (step) {
        case 'prep_fail_save': {
          const db = new DatabaseSync(confined(join(input.workspace, 'noteka.sqlite')));
          try { db.exec(input.enabled ? "CREATE TRIGGER preparation_fail_save BEFORE INSERT ON changes BEGIN SELECT RAISE(ABORT,'headless_save_failure'); END;" : 'DROP TRIGGER preparation_fail_save;'); } finally { db.close(); } return true;
        }
        case 'prep_snapshot': return snapshot(input.workspace);
        case 'prep_operations': return operations.filter(op => op.workspace === input.workspace);
        case 'prep_cli': {
          if (!/^extraction\.(get|list|enqueue|run|control|index|retry|cancel|search)$/.test(input.operation)) throw new Error('Operación sintética no permitida');
          return cli(input.operation, input.input, input.workspace);
        }
        case 'prep_instrument': instruments.set(input.workspace, input.instrument); return true;
        case 'prep_report':
          if (!['dark', 'light'].includes(input.theme) || !input.ok) throw new Error('Reporte inválido');
          writeFileSync(join(root, `preparation-${input.theme}-results.json`), JSON.stringify(input, null, 2)); return true;
        default: throw new Error('Control de preparación desconocido');
      }
    },
  };
}
