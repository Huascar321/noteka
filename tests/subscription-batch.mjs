import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { subscriptionPlan, verifySubscription, minorUnits } from '../scripts/subscription-plan.mjs';
const base = process.env.NOTEKA_TEST_TMP || '/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode';
const binary = resolve(process.env.NOTEKA_CLI || 'target/debug/noteka');
function runner(workspace) {
  return (operation, input = {}, status = 0) => {
    const out = spawnSync(binary, ['--json', '--workspace', workspace, ...operation.split('.')], { input: JSON.stringify(input), encoding: 'utf8' });
    assert.equal(out.status, status, out.stdout); assert.equal(out.stderr, '');
    return JSON.parse(out.stdout);
  };
}
test('16-step offline subscription: preview, exact apply, recost, payment, backup and re-open', () => {
  const root = mkdtempSync(join(base, 'noteka-subscription-'));
  const workspace = join(root, 'w'), call = runner(workspace);
  call('workspace.create', { timezone: 'America/La_Paz' });
  const plan = subscriptionPlan(); assert.equal(plan.steps.length, 16);
  const invalid = structuredClone(plan); invalid.steps[10].input.values.seats = 'invalid';
  assert.equal(call('changes.preview', invalid, 2).ok, false);
  assert.deepEqual(call('collection.list').data.items, []);
  const invalidFormula = structuredClone(plan);
  invalidFormula.steps[7].input.expression.cost_id = '00000000-0000-0000-0000-000000000000';
  invalidFormula.steps[7].bindings = invalidFormula.steps[7].bindings.filter(binding => binding.target !== '/expression/cost_id');
  assert.equal(call('changes.preview', invalidFormula, 2).ok, false);
  assert.deepEqual(call('collection.list').data.items, []);
  const preview = call('changes.preview', plan).data; verifySubscription(preview);
  assert.deepEqual(call('collection.list').data.items, []);
  const applied = call('changes.apply', { plan, expected: preview.receipt }).data;
  const report = verifySubscription(applied);
  assert.deepEqual(applied.steps.map(s => s.exports), preview.steps.map(s => s.exports));
  assert.equal(call('changes.apply', { plan, expected: preview.receipt }, 4).error.details.reason, 'base_hash_mismatch');
  const exports = Object.fromEntries(applied.steps.map(s => [s.key, s.exports]));
  const all = () => call('database.query', { collection_id: report.members, limit: 100 }).data.records.items;
  assert.deepEqual(all(), applied.checks[0].data.records.items);
  const backup = join(root, 'backup'), restored = join(root, 'restored');
  call('workspace.backup', { destination: backup }); call('workspace.restore', { backup, destination: restored });
  const restoredCall = runner(restored);
  assert.deepEqual(restoredCall('database.query', { collection_id: report.members, limit: 100 }).data.records.items, all());
  assert.equal(restoredCall('changes.apply', { plan, expected: preview.receipt }, 2).error.details.reason, 'receipt_scope_mismatch');
  assert.deepEqual(restoredCall('entity.get', { id: report.page_id }).data.document.blocks[0].attrs, { collection_id: report.members, view_id: report.view_id });
  const original = call('record.get', { entity_id: report.subscription_id, collection_id: report.subscriptions }).data;
  call('record.update', { entity_id: report.subscription_id, collection_id: report.subscriptions, expected_revision: original.entity.revision, values: { [exports.cost.property_id]: '150.00' } });
  assert.equal(all().reduce((sum, r) => sum + minorUnits(r.values[exports.amount.property_id]), 0n), 15000n);
  assert.equal(minorUnits(all().find(r => r.values[exports.seats.property_id] === 2).values[exports.amount.property_id]), 5000n);
  const member = all()[0]; call('record.update', { entity_id: member.entity.id, collection_id: report.members, expected_revision: member.entity.revision, values: { [exports.paid.property_id]: true } });
  const pending = call('database.query', { collection_id: report.members, view_id: report.view_id, limit: 100 }).data;
  assert.equal(pending.matching_count, 4); assert(!pending.records.items.some(r => r.entity.id === member.entity.id));
  console.log(JSON.stringify({ evidence: root, total_minor_units: '14569', recost_minor_units: '15000', steps: 16, members: 5, pending_after_payment: 4 }));
});
test('an unrelated writer invalidates construction rather than rebasing it', () => {
  const root = mkdtempSync(join(base, 'noteka-subscription-conflict-')), call = runner(join(root, 'w'));
  call('workspace.create'); const plan = subscriptionPlan(), preview = call('changes.preview', plan).data;
  const writer = call('entity.create', { kind: 'note', title: 'Escritura ajena' }).data;
  assert.equal(call('changes.apply', { plan, expected: preview.receipt }, 4).error.details.reason, 'base_hash_mismatch');
  assert.deepEqual(call('collection.list').data.items, []);
  assert.equal(call('entity.list').data.items[0].id, writer.id);
});
test('the documented construction script uses the real CLI', () => {
  const root = mkdtempSync(join(base, 'noteka-subscription-script-'));
  const report = JSON.parse(execFileSync(process.execPath, ['scripts/subscription-fixture.mjs', join(root, 'w')], { encoding: 'utf8', env: { ...process.env, NOTEKA_CLI: binary } }));
  assert.equal(report.steps, 16); assert.equal(report.total_minor_units, '14569'); assert.equal(report.records.length, 5); assert.equal(report.applied, true);
});
test('allocate tie breaks with the same reserved IDs and cents in preview/apply', () => {
  const root = mkdtempSync(join(base, 'noteka-subscription-ties-')), call = runner(join(root, 'w'));
  call('workspace.create'); const plan = subscriptionPlan(); plan.steps[8].input.values.cost = '0.04';
  const preview = call('changes.preview', plan).data;
  const applied = call('changes.apply', { plan, expected: preview.receipt }).data;
  const amount = preview.steps[7].exports.property_id;
  const values = result => result.checks[0].data.records.items.map(r => [r.entity.id, r.values[amount]]);
  assert.deepEqual(values(preview), values(applied));
  assert.equal(applied.checks[0].data.records.items.reduce((sum, r) => sum + minorUnits(r.values[amount]), 0n), 4n);
  assert.equal(applied.checks[0].data.records.items.filter(r => minorUnits(r.values[amount]) === 0n).length, 1);
});
