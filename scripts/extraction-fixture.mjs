import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { deepStrictEqual, strictEqual } from 'node:assert';

// Sólo sintético: create rechaza destinos ocupados antes de escribir archivos.
const workspace = process.argv[2];
if (!workspace) throw new Error('Uso: node scripts/extraction-fixture.mjs /ruta/nueva/workspace');
const binary = resolve(process.env.NOTEKA_CLI || 'target/debug/noteka');
const call = (group, action, input = {}) => {
  const out = JSON.parse(execFileSync(binary, ['--json', '--workspace', workspace, group, action], { input: JSON.stringify(input), encoding: 'utf8' }));
  if (!out.ok) throw new Error(JSON.stringify(out.error));
  return out.data;
};
call('workspace', 'create', { timezone: 'UTC' });
const text = '\uFEFFLiteral 😀\r\n<script>fetch("https://example.invalid")</script>\n$(touch never-executed)\n'.repeat(4000);
const source = join(workspace, 'synthetic-source.md');
writeFileSync(source, text, { flag: 'wx' });
const attachment = call('attachment', 'import', { source_path: source });
deepStrictEqual(call('extraction', 'list').items, []);
strictEqual(call('extraction', 'control', { action: 'get' }).paused, true);
const selected = call('extraction', 'enqueue', { attachment_id: attachment.id, extractor: 'utf8_text' });
strictEqual(call('extraction', 'run').reason, 'queue_paused');
strictEqual(call('extraction', 'enqueue', { attachment_id: attachment.id, extractor: 'utf8_text' }).job.id, selected.job.id);
call('extraction', 'control', { action: 'set', paused: false, expected_revision: 1 });
const result = call('extraction', 'run', { id: selected.job.id });
strictEqual(result.outcome, 'succeeded');
let cursor; let reconstructed = ''; let pages = 0;
do {
  const page = call('extraction', 'fragments', { id: selected.job.id, limit: 32, ...(cursor ? { cursor } : {}) });
  reconstructed += page.items.map(part => part.text).join('');
  strictEqual(page.generation.complete, true);
  strictEqual(page.covers_generation, !cursor && !page.next_cursor);
  cursor = page.next_cursor; pages++;
} while (cursor);
strictEqual(reconstructed, text);
const retry = call('extraction', 'retry', { id: selected.job.id, expected_revision: result.job.revision });
strictEqual(call('extraction', 'fragments', { id: selected.job.id }).result_from_previous_attempt, true);
const cancelled = call('extraction', 'cancel', { id: selected.job.id, expected_revision: retry.revision });
strictEqual(call('extraction', 'cancel', { id: selected.job.id, expected_revision: cancelled.revision }).revision, cancelled.revision);
deepStrictEqual(call('entity', 'list').items, []);
deepStrictEqual(call('changes', 'list').items, []);
console.log(JSON.stringify({ workspace, attachment_id: attachment.id, job_id: selected.job.id, source_bytes: attachment.bytes, pages, state: cancelled.state, attempts: cancelled.attempts, byte_exact: true }));
