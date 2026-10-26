import { ICONS } from './icons.generated';
export { ICONS };

export type IconName = keyof typeof ICONS;
export const ICON_NAMES = new Set<string>(Object.keys(ICONS));

export const ICON_OPTIONS: { name: IconName; label: string }[] = [
  { name: 'file-text', label: "Document" },
  { name: 'notebook', label: "Notebook" },
  { name: 'home', label: "Home" },
  { name: 'star', label: "Star" },
  { name: 'heart', label: "Heart" },
  { name: 'lightbulb', label: 'Idea' },
  { name: 'book', label: "Book" },
  { name: 'target', label: "Target" },
  { name: 'folder', label: "Folder" },
  { name: 'project', label: "Project" },
  { name: 'task', label: "Task" },
  { name: 'calendar', label: "Calendar" },
  { name: 'code', label: "Code" },
  { name: 'audio', label: 'Audio' },
  { name: 'camera', label: "Camera" },
  { name: 'palette', label: "Design" },
  { name: 'globe', label: 'Web' },
  { name: 'leaf', label: "Nature" },
  { name: 'coffee', label: "Coffee" },
  { name: 'sparkles', label: 'Ideas' },
  { name: 'plane', label: "Travel" },
  { name: 'briefcase', label: "Work" },
  { name: 'graduation', label: "Study" },
  { name: 'wrench', label: "Tools" },
  { name: 'key', label: "Key" },
];

export const ICON_COLORS: { value: string | null; label: string }[] = [
  { value: null, label: "Default" },
  { value: '#e03e3e', label: "Red" },
  { value: '#d9730d', label: "Orange" },
  { value: '#dfab01', label: "Yellow" },
  { value: '#0f7b6c', label: "Green" },
  { value: '#0b6e99', label: "Blue" },
  { value: '#6940a5', label: "Violet" },
  { value: '#ad1a72', label: "Pink" },
  { value: '#787774', label: "Gray" },
];

export function iconName(value: unknown, fallback: IconName = 'file-text'): IconName {
  return typeof value === 'string' && ICON_NAMES.has(value) ? value as IconName : fallback;
}

export function iconColor(value: unknown): string | undefined {
  return typeof value === 'string' && /^#[0-9a-fA-F]{6}$/.test(value) ? value : undefined;
}
