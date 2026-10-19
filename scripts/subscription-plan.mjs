import { randomUUID } from 'node:crypto';

const value = (target, step, name) => ({ kind: 'value', target, source: { step, export: name } });
const property = (target, step) => ({ kind: 'property_key', target, source: { step, export: 'property_id' } });
export function subscriptionPlan(batch_id = randomUUID()) {
  const steps = [];
  const step = (key, operation, input, bindings = []) => steps.push({ key, operation, input, bindings });
  const definition = (key, collection, revision, spec, extra = []) => step(key, 'property.define', { collection_id: null, expected_revision: null, spec }, [value('/collection_id', collection, 'collection_id'), value('/expected_revision', revision, 'collection_revision'), ...extra]);
  step('subscriptions', 'collection.create', { name: 'Subscriptions' });
  step('members', 'collection.create', { name: 'Members' });
  definition('cost', 'subscriptions', 'subscriptions', { name: 'Costo', type: 'money', currency: 'BOB' });
  definition('capacity', 'subscriptions', 'cost', { name: 'Asientos declarados', type: 'number' });
  definition('seats', 'members', 'members', { name: 'Asientos usados', type: 'number' });
  definition('paid', 'members', 'seats', { name: 'Pagó', type: 'checkbox' });
  definition('relation', 'members', 'paid', { name: 'Suscripción', type: 'relation', target_collection_id: null }, [value('/spec/target_collection_id', 'subscriptions', 'collection_id')]);
  step('amount', 'formula.set', { collection_id: null, expected_revision: null, name: 'Importe', expression: { op: 'allocate', relation_id: null, cost_id: null, weight_id: null, scale: 2 } }, [value('/collection_id', 'members', 'collection_id'), value('/expected_revision', 'relation', 'collection_revision'), ...['relation', 'cost', 'seats'].map((key, i) => value(`/expression/${['relation_id', 'cost_id', 'weight_id'][i]}`, key, 'property_id'))]);
  step('subscription', 'record.create', { collection_id: null, title: 'Plan sintético', values: { cost: '145.69', capacity: 6 } }, [value('/collection_id', 'subscriptions', 'collection_id'), property('/values/cost', 'cost'), property('/values/capacity', 'capacity')]);
  for (let i = 0; i < 5; i++) step(`member${i + 1}`, 'record.create', { collection_id: null, title: `Participante ${i + 1}`, values: { seats: i === 0 ? 2 : 1, paid: false, subscription: [null] } }, [value('/collection_id', 'members', 'collection_id'), value('/values/subscription/0', 'subscription', 'entity_id'), property('/values/seats', 'seats'), property('/values/paid', 'paid'), property('/values/subscription', 'relation')]);
  step('pending', 'view.save', { collection_id: null, name: 'Pendientes', layout: 'table', config: { columns: [null, null, null, null], filters: [{ property_id: null, op: 'eq', value: false }], sorts: [{ property_id: null, descending: true }] } }, [value('/collection_id', 'members', 'collection_id'), value('/config/columns/0', 'members', 'title_property_id'), ...['seats', 'amount', 'paid'].map((key, i) => value(`/config/columns/${i + 1}`, key, 'property_id')), value('/config/filters/0/property_id', 'paid', 'property_id'), value('/config/sorts/0/property_id', 'amount', 'property_id')]);
  step('page', 'entity.create', { kind: 'note', title: 'Suscripción compartida', document: { schema_version: 3, blocks: [{ id: 'subscription-pending-view', type: 'collection_view', content: [], attrs: { collection_id: null, view_id: null } }] } }, [value('/document/blocks/0/attrs/collection_id', 'members', 'collection_id'), value('/document/blocks/0/attrs/view_id', 'pending', 'view_id')]);
  const checks = [
    { key: 'all', input: { collection_id: null, limit: 100 }, bindings: [value('/collection_id', 'members', 'collection_id')] },
    { key: 'pending-check', input: { collection_id: null, view_id: null, limit: 100 }, bindings: [value('/collection_id', 'members', 'collection_id'), value('/view_id', 'pending', 'view_id')] },
  ];
  return { version: 1, batch_id, steps, checks };
}
export function minorUnits(value) {
  if (typeof value !== 'string' || !/^\d+(?:\.\d{1,2})?$/.test(value)) throw new Error(`Dinero inválido: ${value}`);
  const [whole, fraction = ''] = value.split('.');
  return BigInt(whole) * 100n + BigInt(fraction.padEnd(2, '0'));
}
export function verifySubscription(result) {
  const steps = Object.fromEntries(result.steps.map(step => [step.key, step]));
  const amount = steps.amount.exports.property_id, seats = steps.seats.exports.property_id, paid = steps.paid.exports.property_id;
  const all = result.checks.find(check => check.key === 'all').data;
  const pending = result.checks.find(check => check.key === 'pending-check').data;
  if (all.matching_count !== 5 || all.records.items.length !== 5 || all.records.next_cursor || pending.matching_count !== 5) throw new Error('Cobertura de miembros incompleta.');
  let total = 0n, used = 0;
  for (const record of all.records.items) {
    if (Object.keys(record.calculation_errors).length) throw new Error(JSON.stringify(record.calculation_errors));
    const weight = record.values[seats], cost = record.values[amount];
    if (record.values[paid] !== false || cost !== (weight === 2 ? '48.57' : '24.28')) throw new Error('Reparto/pago inesperado.');
    total += minorUnits(cost); used += weight;
  }
  if (total !== 14569n || used !== 6 || steps.subscription.data.values[steps.capacity.exports.property_id] !== 6) throw new Error('Centavos/asientos incorrectos.');
  const attrs = steps.page.data.document.blocks[0].attrs;
  if (attrs.collection_id !== steps.members.exports.collection_id || attrs.view_id !== steps.pending.exports.view_id) throw new Error('La página no referencia la vista original.');
  return { workspace: undefined, subscriptions: steps.subscriptions.exports.collection_id, members: steps.members.exports.collection_id, view_id: steps.pending.exports.view_id, page_id: steps.page.exports.entity_id, subscription_id: steps.subscription.exports.entity_id, currency: 'BOB', total_minor_units: total.toString(), records: all.records.items };
}
