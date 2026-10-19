// Sólo pruebas: fixture CLI real, snapshots y mutaciones sintéticas explícitas.
import { spawn } from 'node:child_process';
import { createHash, randomUUID } from 'node:crypto';
import { readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { DatabaseSync } from 'node:sqlite';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
export function createEvidenceControls(root, confined, cli) {
  const fixtures = new Map(), operations = [];
  const fixture = workspace => {
    confined(workspace);
    const value = fixtures.get(workspace);
    if (!value) throw new Error('Workspace de evidencia no registrado');
    return value;
  };
  const data = async (ws, operation, input) => {
    const response = await cli(operation, input, ws);
    if (!response.ok) throw new Error(JSON.stringify(response.error));
    return response.data;
  };
  function snapshot(workspace) {
    fixture(workspace);
    const db = new DatabaseSync(confined(join(workspace, 'noteka.sqlite')), { readOnly: true });
    let logical;
    try {
      const schema = db.prepare('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name').all();
      const tables = Object.fromEntries(db.prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name").all().map(({ name }) => [name,
        db.prepare('SELECT * FROM "' + name.replaceAll('"', '""') + '"').all().map(row => JSON.stringify(row)).sort()]));
      logical = hash(JSON.stringify({ schema, tables }));
    } finally { db.close(); }
    const files = {};
    function walk(directory, prefix = '') {
      for (const entry of readdirSync(confined(directory), { withFileTypes: true })) {
        const path = confined(join(directory, entry.name)), relative = prefix + entry.name;
        if (entry.isSymbolicLink()) throw new Error('Symlink inesperado en fixture');
        if (entry.isDirectory()) walk(path, relative + '/');
        else if (entry.isFile() && !/^noteka\.sqlite(?:-wal|-shm)?$/.test(relative)) files[relative] = hash(readFileSync(path));
      }
    }
    walk(workspace);
    return { logical_sha256: logical, files };
  }
  return {
    async run(workspace, command, work) {
      if (!fixtures.has(workspace)) return work();
      const entry = { workspace, operation: command.operation, input: command.input };
      operations.push(entry);
      const before = command.operation === 'extraction.search' ? snapshot(workspace) : null;
      try { return await work(); }
      finally {
        if (before) {
          entry.readonly_unchanged = JSON.stringify(snapshot(workspace)) === JSON.stringify(before);
          if (!entry.readonly_unchanged) throw new Error('Lectura de evidencia modificó el snapshot sintético');
        }
      }
    },
    async control(step, input) {
      if (step === 'evidence_seed') {
        if (!['dark', 'light'].includes(input.theme)) throw new Error('Tema de evidencia inválido');
        const directory = confined(join(root, `evidence-${input.theme}-${randomUUID()}`));
        await new Promise((done, reject) => {
          const child = spawn('python3', ['-B', resolve('scripts/prepare-evidence-search-fixture.py'), '--root', directory], { env: { ...process.env, NOTEKA_TEST_TMP: root }, stdio: ['ignore', 'pipe', 'pipe'] });
          let stdout = '', stderr = '';
          const timer = setTimeout(() => child.kill(), 120_000);
          child.stdout.on('data', c => { stdout += c; }); child.stderr.on('data', c => { stderr += c; });
          child.on('error', e => { clearTimeout(timer); reject(e); });
          child.on('close', code => { clearTimeout(timer); if (code || stderr) reject(new Error(stderr || stdout)); else done(); });
        });
        const value = JSON.parse(readFileSync(confined(join(directory, 'fixture.json')), 'utf8'));
        if (value.workspace !== join(directory, 'workspace')) throw new Error('Fixture fuera de su workspace');
        fixtures.set(value.workspace, value);
        return value;
      }
      const value = fixture(input.workspace);
      switch (step) {
        case 'evidence_snapshot': return snapshot(input.workspace);
        case 'evidence_operations': return operations.filter(op => op.workspace === input.workspace);
        case 'evidence_query': return cli('extraction.search', input.query, input.workspace);
        case 'evidence_reset': {
          const index = await data(input.workspace, 'extraction.index', { action: 'get' });
          return data(input.workspace, 'extraction.index', { action: 'reset', expected_revision: index.revision });
        }
        case 'evidence_tamper': {
          if (typeof input.altered !== 'boolean') throw new Error('Alteración de prueba inválida');
          const attachment = value.jobs['literal.txt'].attachment_id;
          const path = confined(join(input.workspace, 'attachments', attachment));
          const bytes = readFileSync(confined(join(value.root, 'literal.txt')));
          writeFileSync(path, input.altered ? Buffer.from('fuente alterada sintética') : bytes);
          return true;
        }
        case 'evidence_roundtrip': {
          await data(input.workspace, 'workspace.backup', { destination: confined(join(value.root, 'backup')) });
          const restored = await data(input.workspace, 'workspace.restore', { backup: confined(join(value.root, 'backup')), destination: confined(join(value.root, 'restored')) });
          fixtures.set(restored.path, { ...value, workspace: restored.path });
          return restored;
        }
        case 'evidence_report':
          if (!['dark', 'light'].includes(input.theme) || input.ok !== true || !Array.isArray(input.checks)) throw new Error('Reporte de evidencia inválido');
          writeFileSync(join(root, `evidence-${input.theme}-results.json`), JSON.stringify(input, null, 2));
          return true;
        default: throw new Error('Control de evidencia desconocido');
      }
    },
  };
}
