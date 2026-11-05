import { test } from 'node:test';
import assert from 'node:assert/strict';
import { calendarDays, civilToday, shiftDay, shiftMonth, taskDue, validCivilDate, weekDays } from '../src/lib/project-task-calendar.ts';

test('civil calendar uses workspace timezone and respects leap years and bounds', () => {
  assert.equal(civilToday('America/La_Paz', new Date('2026-01-01T02:30:00Z')), '2025-12-31');
  assert.equal(civilToday('Pacific/Kiritimati', new Date('2026-01-01T02:30:00Z')), '2026-01-01');
  assert.equal(calendarDays('2024-02').filter(Boolean).length, 29);
  assert.equal(calendarDays('2025-02').filter(Boolean).length, 28);
  assert.equal(calendarDays('2026-11')[6], '2026-11-01');
  assert.equal(shiftMonth('0001-01', -1), null);
  assert.equal(shiftMonth('9999-12', 1), null);
  assert.equal(shiftMonth('2026-12', 1), '2027-01');
  assert.equal(calendarDays('0001-01').filter(Boolean).length, 31);
  assert.equal(calendarDays('9999-12').filter(Boolean).length, 31);
  assert.deepEqual(weekDays('2026-12-31'), ['2026-12-28','2026-12-29','2026-12-30','2026-12-31','2027-01-01','2027-01-02','2027-01-03']);
  assert.equal(shiftDay('0001-01-01', -1), null);
  assert.equal(shiftDay('9999-12-31', 1), null);
});

test('due dates distinguish empty from invalid without changing the payload', () => {
  const task = payload => ({payload});
  assert.equal(validCivilDate('2024-02-29'), true);
  assert.equal(validCivilDate('2025-02-29'), false);
  assert.deepEqual(taskDue(task({due_date:'2026-11-05'})), {kind:'date',value:'2026-11-05'});
  assert.deepEqual(taskDue(task({due_date:null})), {kind:'none'});
  assert.deepEqual(taskDue(task({due_date:'2026-02-30'})), {kind:'invalid'});
  assert.deepEqual(taskDue(task(null)), {kind:'invalid'});
});
