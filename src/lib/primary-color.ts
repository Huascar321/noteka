import type { Theme } from './theme';

export const PRIMARY_COLOR_KEY = 'noteka.primary-color.v1';
export const DEFAULT_PRIMARY_COLOR = '#3183a6';
export const PRIMARY_COLORS = [
  { name: 'Blue', value: DEFAULT_PRIMARY_COLOR },
  { name: 'Violet', value: '#8254d4' },
  { name: 'Pink', value: '#c84688' },
  { name: 'Orange', value: '#c97927' },
  { name: 'Green', value: '#298560' },
  { name: 'Teal', value: '#218a8a' },
] as const;
type Store = Pick<Storage, 'getItem' | 'setItem'>;
export type Hsl = { h: number; s: number; l: number };

export function normalizeColor(value: string): string | null {
  return /^#[0-9a-f]{6}$/i.test(value) ? value.toLowerCase() : null;
}
export function readPrimaryColor(store: () => Store): string {
  try { return normalizeColor(store().getItem(PRIMARY_COLOR_KEY) ?? '') ?? DEFAULT_PRIMARY_COLOR; }
  catch { return DEFAULT_PRIMARY_COLOR; }
}
export function savePrimaryColor(color: string, store: () => Store): string {
  const valid = normalizeColor(color);
  if (!valid) return 'Enter a color in #RRGGBB format.';
  try { store().setItem(PRIMARY_COLOR_KEY, valid); return ''; }
  catch { return 'Color applied. Your preference could not be saved on this device.'; }
}

const rgb = (hex: string) => [1, 3, 5].map(offset => parseInt(hex.slice(offset, offset + 2), 16));
const hex = (channels: number[]) => `#${channels.map(value => Math.round(value).toString(16).padStart(2, '0')).join('')}`;
function luminance(color: string): number {
  return rgb(color).map(value => value / 255).map(value => value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4)
    .reduce((total, value, index) => total + value * [0.2126, 0.7152, 0.0722][index], 0);
}
export function colorContrast(first: string, second: string): number {
  const a = luminance(first), b = luminance(second);
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}
function mix(color: string, target: string, amount: number): string {
  const destination = rgb(target);
  return hex(rgb(color).map((channel, index) => channel + (destination[index] - channel) * amount));
}
function readable(color: string, background: string, minimum: number, theme: Theme): string {
  if (colorContrast(color, background) >= minimum) return color;
  const target = theme === 'dark' ? '#ffffff' : '#000000';
  // A bounded search avoids depending on browser-specific CSS color conversion.
  for (let step = 1; step <= 100; step++) {
    const candidate = mix(color, target, step / 100);
    if (colorContrast(candidate, background) >= minimum) return candidate;
  }
  return target;
}
export function primaryPalette(value: string, theme: Theme): Record<string, string> {
  const color = normalizeColor(value) ?? DEFAULT_PRIMARY_COLOR;
  const background = theme === 'dark' ? '#191919' : '#ffffff';
  const surface = theme === 'dark' ? '#202020' : '#f7f7f5';
  // Icons/focus require 3:1; links target 4.5:1. Primary buttons use white text in CSS.
  let accent = readable(color, surface, 3.1, theme);
  const selectionAmount = theme === 'dark' ? 0.22 : 0.12;
  const contrastTarget = theme === 'dark' ? '#ffffff' : '#000000';
  // The active sidebar uses a tinted background as well as the base surface.
  const initialAccent = accent;
  for (let step = 1; step <= 100 && colorContrast(accent, mix(background, accent, selectionAmount)) < 3.1; step++) {
    accent = mix(initialAccent, contrastTarget, step / 100);
  }
  const text = colorContrast(accent, '#000000') > colorContrast(accent, '#ffffff') ? '#000000' : '#ffffff';
  const link = readable(color, surface, 4.6, theme);
  const hover = mix(accent, text, 0.10);
  return {
    '--primary-color': color, '--accent': accent, '--accent-text': text,
    '--glass-action-tint': readable(color, '#ffffff', 7, 'light'),
    '--accent-hover': colorContrast(hover, text) >= 4.5 ? hover : accent, '--focus': link,
    '--link': link, '--section-icon': accent,
    '--selected': mix(background, accent, selectionAmount),
    '--section-bg': mix(background, accent, theme === 'dark' ? 0.22 : 0.10),
    '--section-text': theme === 'dark' ? '#e9e9e7' : '#37352f',
  };
}
export function hexToHsl(color: string): Hsl {
  const [r, g, b] = rgb(normalizeColor(color) ?? DEFAULT_PRIMARY_COLOR).map(value => value / 255);
  const max = Math.max(r, g, b), min = Math.min(r, g, b), d = max - min;
  const l = (max + min) / 2;
  let h = 0;
  if (d) h = max === r ? ((g - b) / d + (g < b ? 6 : 0)) * 60 : max === g ? ((b - r) / d + 2) * 60 : ((r - g) / d + 4) * 60;
  return { h, s: d ? d / (1 - Math.abs(2 * l - 1)) * 100 : 0, l: l * 100 };
}
export function hslToHex({ h, s, l }: Hsl): string {
  const lightness = l / 100, saturation = s / 100;
  const a = saturation * Math.min(lightness, 1 - lightness);
  return hex([0, 8, 4].map(offset => {
    const k = (offset + h / 30) % 12;
    return (lightness - a * Math.max(-1, Math.min(k - 3, 9 - k, 1))) * 255;
  }));
}
