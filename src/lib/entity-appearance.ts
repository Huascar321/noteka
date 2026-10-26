import type { EntityKind } from './contracts.generated';
import { iconColor, type IconName } from './icon-set';
import { visualIconName } from './visual-icon';

const DEFAULT_ICON: Record<EntityKind, IconName> = { note: 'file-text', task: 'task', project: 'project', tag: 'tag', person: 'person' };
const TITLE_ICONS: Record<string, IconName | undefined> = { home: 'home', notes: 'notebook', tasks: 'task', projects: 'project' };

export function entityIcon(kind: EntityKind, title = '', payload?: unknown): string {
  if (payload && typeof payload === 'object' && 'icon' in payload && typeof payload.icon === 'string') return visualIconName(payload.icon, DEFAULT_ICON[kind]);
  if (kind === 'note' && payload && typeof payload === 'object') {
    if ('goal' in payload && payload.goal) return 'trophy';
    if ('milestone' in payload && payload.milestone) return 'target';
  }
  const known = TITLE_ICONS[title.trim().toLowerCase()];
  return known ?? DEFAULT_ICON[kind];
}

export function entityIconColor(payload?: unknown): string | undefined {
  return payload && typeof payload === 'object' && 'icon_color' in payload ? iconColor(payload.icon_color) : undefined;
}
