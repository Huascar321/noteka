import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { deepStrictEqual } from 'node:assert';
import { subscriptionPlan, verifySubscription } from './subscription-plan.mjs';

// Exclusivamente sintético. workspace.create rechaza destinos existentes.
const workspace = process.argv[2];
if (!workspace) throw new Error('Uso: node scripts/subscription-fixture.mjs /ruta/nueva/al/workspace');
const binary = resolve(process.env.NOTEKA_CLI || 'target/debug/noteka');
const call = (group, action, input = {}) => {
  const out = JSON.parse(execFileSync(binary, ['--json', '--workspace', workspace, group, action], { input: JSON.stringify(input), encoding: 'utf8' }));
  if (!out.ok) throw new Error(JSON.stringify(out.error));
  return out.data;
};
call('workspace', 'create', { timezone: 'America/La_Paz' });
const plan = subscriptionPlan();
const preview = call('changes', 'preview', plan);
verifySubscription(preview); // Evalúa centavos antes de confirmar; no cambia el plan.
deepStrictEqual(call('collection', 'list').items, []);
const applied = call('changes', 'apply', { plan, expected: preview.receipt });
deepStrictEqual(applied.steps.map(step => step.exports), preview.steps.map(step => step.exports));
const report = verifySubscription(applied);
const saved = call('database', 'query', { collection_id: report.members, view_id: report.view_id, limit: 100 });
deepStrictEqual(saved.records, applied.checks[1].data.records);
deepStrictEqual(call('entity', 'get', { id: report.page_id }).document, applied.steps[15].data.document);
console.log(JSON.stringify({ ...report, workspace, batch_id: plan.batch_id, steps: plan.steps.length, applied: applied.applied }, null, 2));
