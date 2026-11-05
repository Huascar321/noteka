import { sceneNoise, type KoiPose } from './ascii-cover-scene';

const CW = 6, CH = 8;
const GLYPHS = ['.', 'o', 'O', '|', '/', '\\', '~'];
const DARK = ['#82969e', '#789a7b', '#8baab5'];
const LIGHT = ['#697f87', '#648464', '#6e909e'];

export function backgroundKoiPoses(width: number, height: number, seconds: number, seed: number): { pose: KoiPose; opacity: number }[] {
  const baseLength = Math.min(width * .12, height * .28, 80), margin = Math.max(70, baseLength);
  return Array.from({ length: width < 600 ? 5 : 8 }, (_, i) => {
    const direction = i % 2 ? -1 : 1, initial = sceneNoise(seed, i + 201), depth = sceneNoise(seed, i + 205);
    const speed = 5 + sceneNoise(seed, i + 202) * 5, range = width + margin * 2;
    const progress = (initial + seconds * speed / range) % 1;
    const length = baseLength * (1.12 - depth * .28), x = (direction > 0 ? progress : 1 - progress) * range - margin;
    const wave = seconds * .32 + initial * 6;
    const y = height * [.16, .77, .29, .86][i % 4] + (sceneNoise(seed, i + 203) - .5) * height * .06 + Math.sin(wave) * 3;
    const fade = Math.max(0, Math.min(1, (x + length * .6) / (length * .9), (width + length * .6 - x) / (length * .9)));
    return { opacity: (.46 - depth * .14) * fade, pose: {
      x, y, length, heading: Math.atan2(Math.cos(wave) * .96, direction * speed),
      phase: seconds * (4.1 + i * .17) + initial * Math.PI * 2, seed: seed + 4001 + i * 193
    } };
  });
}

class HabitatAtlas {
  private canvas = document.createElement('canvas');
  private indices = new Map(GLYPHS.map((glyph, index) => [glyph, index]));

  constructor(readonly dpr: number, dark: boolean) {
    this.canvas.width = GLYPHS.length * CW * dpr;
    this.canvas.height = 3 * CH * dpr;
    const ctx = this.canvas.getContext('2d');
    if (!ctx) throw Error('Canvas2D unavailable');
    ctx.scale(dpr, dpr);
    ctx.font = '400 8px ui-monospace, SFMono-Regular, Menlo, monospace';
    ctx.textBaseline = 'alphabetic';
    (dark ? DARK : LIGHT).forEach((color, row) => {
      ctx.fillStyle = color;
      GLYPHS.forEach((glyph, column) => ctx.fillText(glyph, column * CW + .5, row * CH + 6.5));
    });
  }

  text(ctx: CanvasRenderingContext2D, text: string, x: number, y: number, tone: number, opacity: number) {
    ctx.globalAlpha = opacity;
    for (let i = 0; i < text.length; i++) {
      const index = this.indices.get(text[i]);
      if (index !== undefined) ctx.drawImage(this.canvas, index * CW * this.dpr, tone * CH * this.dpr,
        CW * this.dpr, CH * this.dpr, x + i * CW, y, CW, CH);
    }
  }
}

// This layer uses the cover's active time. It owns no animation loop or listeners.
export class KoiHabitatRenderer {
  private atlas: HabitatAtlas | null = null;
  private dark = true;

  constructor(private seed: number) {}

  draw(ctx: CanvasRenderingContext2D, width: number, height: number, seconds: number, dpr: number, dark: boolean) {
    if (!width || !height) return;
    if (!this.atlas || this.atlas.dpr !== dpr || this.dark !== dark) this.atlas = new HabitatAtlas(dpr, dark);
    this.dark = dark;
    const atlas = this.atlas, narrow = width < 600;

    // Sparse fronds stay rooted at the lower edge and sway towards their tips.
    const plants = narrow ? 4 : 7;
    for (let i = 0; i < plants; i++) {
      const base = width * (.065 + i * .87 / (plants - 1) + (sceneNoise(this.seed, i + 140) - .5) * .035);
      const phase = sceneNoise(this.seed, i + 141) * Math.PI * 2;
      const opacity = (dark ? .24 : .30) + sceneNoise(this.seed, i + 142) * .09;
      for (let stem = 0; stem < 3; stem++) {
        const length = Math.min(height * .28, 78) * (.66 + sceneNoise(this.seed, i * 7 + stem + 151) * .4);
        const steps = Math.max(3, Math.round(length / CH));
        const sway = (u: number) => Math.sin(seconds * .32 + phase + stem * .55 - u * 3.4) * 9 * u;
        for (let n = 0; n <= steps; n++) {
          const u = n / steps, x = base + (stem - 1) * 7 + sway(u), lean = sway(u) - sway(Math.max(0, (n - 1) / steps));
          const y = height - CH - n * CH, glyph = n === steps ? '~' : lean > 1 ? '/' : lean < -1 ? '\\' : '|';
          atlas.text(ctx, glyph, x, y, 1, opacity);
          if (n > 1 && n % 3 === 0) {
            const side = (n + stem) % 2 ? -1 : 1;
            atlas.text(ctx, side < 0 ? '\\' : '/', x + side * CW, y, 1, opacity * .8);
          }
        }
      }
    }

    // Bubble cycles fade at both ends instead of popping back into the frame.
    for (let i = 0; i < (narrow ? 6 : 12); i++) {
      const phase = sceneNoise(this.seed, i + 241), period = 20 + sceneNoise(this.seed, i + 242) * 20;
      const progress = (phase + seconds / period) % 1;
      const x = width * (.05 + sceneNoise(this.seed, i + 243) * .9) + Math.sin(seconds * .35 + phase * 6) * 5;
      const y = height + 12 - progress * (height + 36);
      const fade = Math.min(1, progress * 8, (1 - progress) * 8);
      atlas.text(ctx, i % 4 === 0 ? 'O' : i % 3 === 0 ? '.' : 'o', x, y, 2,
        fade * (.16 + sceneNoise(this.seed, i + 244) * .10));
    }
    ctx.globalAlpha = 1;
  }
}
