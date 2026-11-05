import type { Entity } from './contracts.generated';

function utcDate(year: number, month: number, day: number): Date {
  const value = new Date(0);
  value.setUTCHours(12, 0, 0, 0);
  value.setUTCFullYear(year, month - 1, day);
  return value;
}
export function civilToday(timezone: string, now = new Date()): string {
  const parts = new Intl.DateTimeFormat('en-CA', { timeZone: timezone, year: 'numeric', month: '2-digit', day: '2-digit' }).formatToParts(now);
  const part = (type: string) => parts.find(p => p.type === type)?.value ?? '';
  return `${part('year').padStart(4, '0')}-${part('month')}-${part('day')}`;
}
export function validCivilDate(value: unknown): value is string {
  if (typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const [year, month, day] = value.split('-').map(Number);
  if (year < 1 || year > 9999 || month < 1 || month > 12 || day < 1) return false;
  const date = utcDate(year, month, day);
  return date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day;
}
export function shiftMonth(month: string, delta: number): string | null {
  if (!/^\d{4}-\d{2}$/.test(month)) return null;
  const [year, value] = month.split('-').map(Number);
  if (year < 1 || year > 9999 || value < 1 || value > 12) return null;
  const absolute = (year - 1) * 12 + value - 1 + delta;
  if (absolute < 0 || absolute >= 9999 * 12) return null;
  return `${String(Math.floor(absolute / 12) + 1).padStart(4, '0')}-${String(absolute % 12 + 1).padStart(2, '0')}`;
}
export function shiftDay(value: string, delta: number): string | null {
  if (!validCivilDate(value)) return null;
  const [year, month, day] = value.split('-').map(Number);
  const date = utcDate(year, month, day);
  date.setUTCDate(date.getUTCDate() + delta);
  if (date.getUTCFullYear() < 1 || date.getUTCFullYear() > 9999) return null;
  return date.toISOString().slice(0, 10);
}
export function weekDays(value: string): (string | null)[] {
  if (!validCivilDate(value)) return [];
  const [year, month, day] = value.split('-').map(Number);
  const offset = (utcDate(year, month, day).getUTCDay() + 6) % 7;
  return Array.from({ length: 7 }, (_, index) => shiftDay(value, index - offset));
}
export function calendarDays(month: string): (string | null)[] {
  const [year, value] = month.split('-').map(Number);
  if (!shiftMonth(month, 0)) return [];
  const first = utcDate(year, value, 1), start = (first.getUTCDay() + 6) % 7;
  const last = utcDate(year, value + 1, 0).getUTCDate();
  return Array.from({ length: Math.ceil((start + last) / 7) * 7 }, (_, i) => {
    const day = i - start + 1;
    return day > 0 && day <= last ? `${month}-${String(day).padStart(2, '0')}` : null;
  });
}
export function taskDue(task: Entity): { kind: 'date'; value: string } | { kind: 'none' | 'invalid' } {
  const payload = task.payload;
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) return { kind: 'invalid' };
  const due = (payload as Record<string, unknown>).due_date;
  if (due == null) return { kind: 'none' };
  return validCivilDate(due) ? { kind: 'date', value: due } : { kind: 'invalid' };
}
