export function goalDate(value?: string | null, timezone = 'UTC', civil = false): string {
  if (!value) return '';
  const date = new Date(civil ? `${value}T12:00:00Z` : value);
  if (!Number.isFinite(date.getTime())) return '';
  return new Intl.DateTimeFormat('en-US', {
    timeZone: civil ? 'UTC' : timezone, month: 'long', day: 'numeric', year: 'numeric',
    ...(civil ? {} : { hour: 'numeric' as const, minute: '2-digit' as const }),
  }).format(date).replace(' at ', ' ');
}
