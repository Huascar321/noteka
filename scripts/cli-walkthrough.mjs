import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';

const root = process.env.NOTEKA_DEMO_ROOT || mkdtempSync(join(tmpdir(), 'noteka-demo-'));
const workspace = join(root, 'workspace');
function cli(group, action, input = {}) {
  const result = JSON.parse(execFileSync(resolve('target/debug/noteka'), ['--workspace', workspace, '--json', group, action], { input: JSON.stringify(input), encoding: 'utf8' }));
  assert.equal(result.ok, true); return result.data;
}
assert.equal(cli('operations', 'describe').api_version, 1);
const info = cli('workspace', 'create', { timezone: 'America/La_Paz' });
assert.equal(cli('workspace', 'open').id, info.id);
let note = cli('entity', 'create', JSON.parse(readFileSync('tests/fixtures/note.json', 'utf8')));
const project = cli('entity', 'create', { kind: 'project', title: 'CRE sintético' });
const result = cli('relation', 'add', { source_id: note.id, target_id: project.id, kind: 'project', expected_revision: note.revision });
note = result.entity;
assert.equal(cli('relation', 'list', { entity_id: project.id }).items[0].id, result.relation.id);
note = cli('entity', 'update', { id: note.id, expected_revision: note.revision, title: 'Nota actualizada' });
note = cli('entity', 'archive', { id: note.id, expected_revision: note.revision });
assert.equal(cli('entity', 'list', { kind: 'note' }).items.length, 0);
note = cli('entity', 'restore', { id: note.id, expected_revision: note.revision });
assert.equal(cli('changes', 'list', { entity_id: note.id }).items.length, 5);
const backup = join(root, 'backup'); cli('workspace', 'backup', { destination: backup });
const restored = cli('workspace', 'restore', { backup, destination: join(root, 'restored') });
assert.equal(restored.id, info.id);
const recovered = JSON.parse(execFileSync(resolve('target/debug/noteka'), ['--json', '--workspace', restored.path, 'entity', 'get'], { input: JSON.stringify({ id: note.id }), encoding: 'utf8' })).data;
assert.equal(recovered.revision, note.revision); assert.deepEqual(recovered.document, note.document);
console.log(`Recorrido CLI verificado; fixtures sintéticas: ${root}`);
