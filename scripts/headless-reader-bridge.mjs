// Sólo pruebas: sirve un build headless y workspaces sintéticos registrados.
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { createHash, randomUUID } from 'node:crypto';
import { mkdtempSync, readFileSync, writeFileSync, readdirSync, realpathSync, lstatSync } from 'node:fs';
import { join, resolve, dirname, extname } from 'node:path';
import { tmpdir } from 'node:os';
import { DatabaseSync } from 'node:sqlite';
const hash = data => createHash('sha256').update(data).digest('hex');
const readonly = new Set(['workspace.open', 'entity.get', 'entity.list', 'relation.list', 'workflow.query', 'home.query', 'extraction.fragments', 'extraction.pdf_fragments', 'extraction.xlsx_cells', 'extraction.ocr_regions']);
export async function startReaderBridge(dist, isolated) {
  const root = realpathSync(mkdtempSync(join(process.env.NOTEKA_TEST_TMP || tmpdir(), 'noteka-reader-headless-')));
  const fixtures = new Map(), operations = [], instruments = new Map(), altered = new Map();
  const binary = resolve('target/debug/noteka');
  function confined(path) {
    if (typeof path !== 'string') throw new Error('Ruta requerida');
    const absolute = resolve(path); if (!absolute.startsWith(root + '/')) throw new Error('Ruta fuera del corpus');
    let parent = absolute;
    while (!lstatSync(parent, { throwIfNoEntry: false })) parent = dirname(parent);
    const actual = realpathSync(parent); if (actual !== root && !actual.startsWith(root + '/')) throw new Error('Symlink fuera del corpus');
    return absolute;
  }
  const fixture = path => { confined(path); const f = fixtures.get(path); if (!f) throw new Error('Workspace no registrado'); return f; };
  const cli = (ws, operation, input) => new Promise((done, reject) => {
    confined(ws);
    if (!/^[a-z_]+\.[a-z_]+$/.test(operation)) throw new Error('Operación inválida');
    const args = ['--workspace', ws, '--json', ...operation.split('.').map(part => part.replaceAll('_', '-'))];
    const sandbox = process.platform === 'darwin';
    const child = spawn(sandbox ? '/usr/bin/sandbox-exec' : binary, sandbox ? ['-p', '(version 1)(allow default)(deny network*)', binary, ...args] : args, { stdio: ['pipe', 'pipe', 'pipe'] });
    child.stdout.setEncoding('utf8'); child.stderr.setEncoding('utf8');
    let stdout = '', stderr = ''; child.stdout.on('data', c => stdout += c); child.stderr.on('data', c => stderr += c); child.on('error', reject);
    child.on('close', code => { try { const r = JSON.parse(stdout); if (stderr || typeof r.ok !== 'boolean' || r.ok !== (code === 0)) throw new Error(stderr || 'Envelope CLI inválido'); done(r); } catch (error) { reject(error); } });
    child.stdin.end(JSON.stringify(input));
  });
  function snapshot(ws) {
    fixture(ws); const db = new DatabaseSync(confined(join(ws, 'noteka.sqlite')), { readOnly: true });
    try {
      const schema = db.prepare('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name').all();
      const tables = Object.fromEntries(db.prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name").all().map(({ name }) => [name, db.prepare('SELECT * FROM "' + name.replaceAll('"', '""') + '"').all().map(row => JSON.stringify(row)).sort()]));
      const files = Object.fromEntries(readdirSync(confined(join(ws, 'attachments'))).map(name => [name, hash(readFileSync(confined(join(ws, 'attachments', name))))]));
      return { logical: hash(JSON.stringify({ schema, tables })), files };
    } finally { db.close(); }
  }
  async function control(step, input) {
    if (step === 'reader_seed') {
      if (!['dark', 'light'].includes(input.theme)) throw new Error('Tema inválido');
      const directory = confined(join(root, 'reader-' + input.theme + '-' + randomUUID()));
      await new Promise((done, reject) => {
        const child = spawn('python3', ['-B', resolve('scripts/prepare-attachment-reader-fixture.py'), '--root', directory], { env: { ...process.env, NOTEKA_TEST_TMP: root }, stdio: ['ignore', 'pipe', 'pipe'] });
        let out = '', err = ''; child.stdout.on('data', c => out += c); child.stderr.on('data', c => err += c); child.on('error', reject); child.on('close', code => code || err ? reject(new Error(err || out)) : done());
      });
      const f = JSON.parse(readFileSync(confined(join(directory, 'fixture.json')), 'utf8'));
      if (f.workspace !== join(directory, 'workspace')) throw new Error('Workspace de fixture inválido');
      fixtures.set(f.workspace, f); return { ...f, isolated };
    }
    const f = fixture(input.workspace);
    switch (step) {
      case 'reader_snapshot': return snapshot(input.workspace);
      case 'reader_operations': {
        const selected=operations.filter(o => o.workspace===input.workspace);
        writeFileSync(join(root,'reader-operations-inspection.json'),JSON.stringify(selected,null,2));return selected;
      }
      case 'reader_instrument': instruments.set(input.workspace, input.instrument); return true;
      case 'reader_cli': {
        if (!['extraction.get', 'extraction.retry', 'extraction.run', 'extraction.index'].includes(input.operation)) throw new Error('Operación de prueba inválida');
        return cli(input.workspace, input.operation, input.input);
      }
      case 'reader_tamper': {
        const source = f.fixtures[input.name]; if (!source || typeof input.enabled !== 'boolean') throw new Error('Fuente de prueba inválida');
        const original = confined(join(input.workspace, 'attachments', source.attachment.id));
        writeFileSync(original, input.enabled ? Buffer.from('Alteración sintética') : readFileSync(confined(join(f.root, input.name)))); return true;
      }
      case 'reader_corrupt': {
        const source = f.fixtures['literal.txt'], db = new DatabaseSync(confined(join(input.workspace, 'noteka.sqlite')));
        try {
          const generation = source.job.generation.id;
          if (input.enabled) {
            if (altered.has(input.workspace)) throw new Error('Ya alterado');
            const row = db.prepare('SELECT text FROM extraction_fragments WHERE generation_id=? AND ordinal=0').get(generation);
            altered.set(input.workspace, row.text); db.prepare('UPDATE extraction_fragments SET text=? WHERE generation_id=? AND ordinal=0').run('x'.repeat(Buffer.byteLength(row.text)), generation);
          } else {
            if (!altered.has(input.workspace)) throw new Error('No alterado');
            db.prepare('UPDATE extraction_fragments SET text=? WHERE generation_id=? AND ordinal=0').run(altered.get(input.workspace), generation); altered.delete(input.workspace);
          }
        } finally { db.close(); } return true;
      }
      case 'reader_report':
        if (!['dark', 'light'].includes(input.theme) || !input.ok) throw new Error('Reporte inválido');
        writeFileSync(join(root, `reader-${input.theme}-results.json`), JSON.stringify(input, null, 2)); return true;
      default: throw new Error('Control de lector desconocido');
    }
  }
  const server = createServer(async (req, res) => {
    try {
      const origin = 'http://127.0.0.1:' + server.address().port;
      if (req.headers.host !== new URL(origin).host || req.headers.origin && req.headers.origin !== origin) throw new Error('Origen inválido');
      if (req.method === 'POST') {
        if (req.headers['content-type'] !== 'application/json') throw new Error('JSON requerido');
        req.setEncoding('utf8');
        let raw = ''; for await (const part of req) { raw += part; if (raw.length > 3 * 1024 * 1024) throw new Error('Request excesivo'); }
        const input = JSON.parse(raw); let result;
        if (req.url === '/__test__/control') result = await control(input.step, input.input || {});
        else if (req.url === '/__test__/operate') {
          const ws = input.workspace, command = input.command; fixture(ws);
          if (!readonly.has(command.operation)) throw new Error('Este puente de lector sólo permite lecturas');
          const before = snapshot(ws), entry = { workspace: ws, operation: command.operation, input: command.input }; operations.push(entry);
          const instrument = instruments.get(ws);
          if (instrument?.operation === command.operation && instrument.delay) await new Promise(r => setTimeout(r, instrument.delay));
          if (instrument?.operation === command.operation && instrument.failure) result = { api_version: 1, request_id: randomUUID(), ok: false, error: { code: 'storage_error', message: '<script>Fallo lector sintético</script>', details: { reason: 'headless_reader_injected' } } };
          result ??= await cli(ws, command.operation, command.input);
          entry.readonly_unchanged = JSON.stringify(snapshot(ws)) === JSON.stringify(before);
          writeFileSync(join(root, 'reader-operations.json'), JSON.stringify(operations, null, 2));
          if (!entry.readonly_unchanged) { entry.before=before;entry.after=snapshot(ws);writeFileSync(join(root,'reader-write-diagnostic.json'),JSON.stringify(entry,null,2));throw new Error('Lectura UI modificó el workspace'); }
        } else throw new Error('Ruta POST inválida');
        res.writeHead(200, { 'content-type': 'application/json' }); res.end(JSON.stringify(result)); return;
      }
      const path = resolve(dist, '.' + (req.url === '/' || req.url.startsWith('/?') ? '/index.html' : req.url.split('?')[0]));
      if (!path.startsWith(resolve(dist) + '/')) throw new Error('Ruta GET inválida');
      res.setHeader('content-type', ({ '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' })[extname(path)] || 'application/octet-stream'); res.end(readFileSync(path));
    } catch (error) { res.writeHead(500, { 'content-type': 'application/json' }); res.end(JSON.stringify({ error: String(error) })); }
  });
  await new Promise((done, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', done); });
  return { root, origin: 'http://127.0.0.1:' + server.address().port, close: () => new Promise((done, reject) => server.close(error => error ? reject(error) : done())) };
}
