import { DEFAULT_PRIMARY_COLOR, hexToHsl, hslToHex, normalizeColor } from './primary-color.ts';

// Koi take the accent hue at half its saturation and a restrained light-to-dark
// range per theme, so the marks read as a discreet tint instead of a brand fill.
const KOI_LIGHTNESS = { dark: [26, 66], light: [36, 64] } as const;
const KOI_SATURATION = .5;
export type KoiPalette = readonly [dark: string, light: string];

export function koiPalette(primary: string, dark: boolean): KoiPalette {
  const { h, s } = hexToHsl(normalizeColor(primary) ?? DEFAULT_PRIMARY_COLOR);
  const [low, high] = dark ? KOI_LIGHTNESS.dark : KOI_LIGHTNESS.light;
  return [hslToHex({ h, s: s * KOI_SATURATION, l: low }), hslToHex({ h, s: s * KOI_SATURATION, l: high })];
}
