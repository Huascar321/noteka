import { sceneNoise, type JellyPose } from './ascii-cover-scene';

export function backgroundJellyPoses(width: number, height: number, seconds: number, seed: number): { pose: JellyPose; opacity: number }[] {
  const radius = Math.min(width * .05, height * .115, 32);
  return Array.from({ length: width < 600 ? 3 : 5 }, (_, i) => {
    const initial = sceneNoise(seed, i + 401) * Math.PI * 2, depth = sceneNoise(seed, i + 405);
    return { opacity: .42 - depth * .12, pose: {
      x: width * [.10, .58, .90, .32, .77][i] + Math.sin(seconds * .11 + initial) * width * .016,
      y: height * [.20, .20, .30, .26, .12][i] + Math.sin(seconds * .17 + initial) * height * .03,
      radius: radius * (1 - depth * .22), angle: Math.sin(seconds * .22 + initial) * .14,
      pulse: .5 + .5 * Math.sin(seconds * 1.5 + initial), phase: seconds * 1.3 + initial,
      family: i % 2 ? 3 : 2, seed: seed + 5001 + i * 317
    } };
  });
}

const CW = 6, CH = 8, GLYPHS = ['.', ':', 'o', 'O', '+', '/', '\\', '|', '-'];
const DARK = ['#759aa7', '#9a83b1'];
const LIGHT = ['#638a97', '#88729e'];
type HabitatCell = { x: number; y: number; glyph: number; tone: number; opacity: number; phase: number; sway: number };

export class JellyHabitatRenderer {
  private canvas = document.createElement('canvas');
  private dpr = 0;
  private dark = true;
  private width = 0;
  private height = 0;
  private corals: HabitatCell[] = [];
  private rays: HabitatCell[] = [];

  constructor(private seed: number) {}

  private prepareBackdrop(width: number, height: number) {
    this.width = width; this.height = height; this.corals = []; this.rays = [];
    const radius = Math.min(width * .12, height * .29, 82);
    // Each fan shares a small stem and branching ribs; cache cells, not strokes.
    for (let fan = 0; fan < 2; fan++) {
      const origin = width * (fan ? .965 : .045), base = height + 5;
      const scale = radius * (fan ? .84 : 1), phase = sceneNoise(this.seed, fan + 541) * Math.PI * 2;
      const cells = new Map<string, HabitatCell>();
      const branch = (x1: number, y1: number, x2: number, y2: number) => {
        const dx = x2 - x1, dy = y2 - y1, steps = Math.ceil(Math.hypot(dx, dy) / 3);
        const glyph = Math.abs(dx) < Math.abs(dy) * .35 ? 7 : Math.abs(dy) < Math.abs(dx) * .35 ? 8 : dx * dy < 0 ? 5 : 6;
        for (let step = 0; step <= steps; step++) {
          const u = steps ? step / steps : 0, x = Math.round((x1 + dx * u) / CW) * CW, y = Math.round((y1 + dy * u) / CH) * CH;
          if (x < -CW || x > width || y < 0 || y >= height) continue;
          const rise = Math.max(0, (base - y) / scale);
          cells.set(`${x},${y}`, { x, y, glyph, tone: fan ? 0 : 1, opacity: .24 + rise * .10, phase, sway: rise * rise * 2.4 });
        }
      };
      branch(origin, base, origin, base - scale * .22);
      for (let rib = 0; rib < 7; rib++) {
        const angle = (rib / 6 - .5) * 2.25, length = scale * (.88 + sceneNoise(this.seed, rib + fan * 11 + 551) * .12);
        const point = (u: number) => ({ x: origin + Math.sin(angle) * length * u ** 1.15, y: base - scale * .17 - Math.cos(angle) * length * u });
        for (let segment = 0; segment < 4; segment++) {
          const a = point(segment / 4), b = point((segment + 1) / 4); branch(a.x, a.y, b.x, b.y);
          if (segment > 0 && segment < 3) {
            const spread = scale * .15, sign = rib < 3 ? -1 : 1;
            branch(b.x, b.y, b.x + sign * spread, b.y - scale * .14);
          }
        }
        if (rib < 6) for (const u of [.48, .75]) {
          const p = point(u), nextAngle = angle + 2.25 / 6;
          branch(p.x, p.y, origin + Math.sin(nextAngle) * length * u ** 1.15, base - scale * .17 - Math.cos(nextAngle) * length * u);
        }
      }
      this.corals.push(...cells.values());
    }
    // Sparse diagonal shafts remain at the sides, with a soft depth fade.
    for (let ray = 0; ray < (width < 600 ? 2 : 3); ray++) {
      const origin = width * [ .015, .79, .925 ][ray], phase = sceneNoise(this.seed, ray + 601) * Math.PI * 2;
      for (let y = 0; y < height * .76; y += CH) {
        const depth = y / (height * .76), center = origin + y * .24, halfWidth = 7 + y * .13;
        for (let x = Math.floor((center - halfWidth) / CW) * CW; x <= center + halfWidth; x += CW) {
          if (x < 0 || x >= width || sceneNoise(this.seed, x / CW + y / CH * 997 + ray * 1701) > .58) continue;
          const edge = Math.max(0, 1 - Math.abs(x - center) / halfWidth);
          this.rays.push({ x, y, glyph: (x / CW + y / CH) % 5 === 0 ? 1 : 0, tone: 0,
            opacity: .22 * edge * (1 - depth) ** 1.35, phase, sway: depth * 3 });
        }
      }
    }
  }

  draw(ctx: CanvasRenderingContext2D, width: number, height: number, seconds: number, dpr: number, dark: boolean) {
    if (!width || !height) return;
    if (width !== this.width || height !== this.height) this.prepareBackdrop(width, height);
    if (dpr !== this.dpr || dark !== this.dark) {
      this.dpr = dpr; this.dark = dark;
      this.canvas.width = GLYPHS.length * CW * dpr;
      this.canvas.height = 2 * CH * dpr;
      const atlas = this.canvas.getContext('2d');
      if (!atlas) throw Error('Canvas2D unavailable');
      atlas.scale(dpr, dpr);
      atlas.font = '400 8px ui-monospace, SFMono-Regular, Menlo, monospace';
      atlas.textBaseline = 'alphabetic';
      (dark ? DARK : LIGHT).forEach((color, row) => {
        atlas.fillStyle = color;
        GLYPHS.forEach((glyph, index) => atlas.fillText(glyph, index * CW + .5, row * CH + 6.5));
      });
    }
    const glyph = (index: number, tone: number, x: number, y: number, opacity: number) => {
      ctx.globalAlpha = opacity;
      ctx.drawImage(this.canvas, index * CW * dpr, tone * CH * dpr, CW * dpr, CH * dpr, x, y, CW, CH);
    };
    for (const cell of this.rays) glyph(cell.glyph, cell.tone, cell.x + Math.sin(seconds * .10 + cell.phase) * cell.sway, cell.y,
      cell.opacity * (.86 + Math.sin(seconds * .16 + cell.phase) * .14));
    for (const cell of this.corals) glyph(cell.glyph, cell.tone, cell.x + Math.sin(seconds * .20 + cell.phase) * cell.sway, cell.y, cell.opacity);
    // Distant plankton and bubbles follow the same active time as the jellyfish.
    for (let i = 0; i < (width < 600 ? 18 : 36); i++) {
      const phase = sceneNoise(this.seed, i + 441), speed = 1 + sceneNoise(this.seed, i + 442) * 1.7;
      const progress = (phase + seconds * speed / (height + 32)) % 1, y = height + 16 - progress * (height + 32);
      const x = sceneNoise(this.seed, i + 443) * width + Math.sin(seconds * .08 + phase * 6) * 6;
      const fade = Math.max(0, Math.min(1, (y + 8) / 18, (height + 8 - y) / 18));
      glyph(i % 9 === 0 ? 4 : i % 3 === 0 ? 1 : 0, i % 2, x, y, fade * (.10 + sceneNoise(this.seed, i + 444) * .08));
    }
    for (let i = 0; i < (width < 600 ? 5 : 10); i++) {
      const phase = sceneNoise(this.seed, i + 481), period = 24 + sceneNoise(this.seed, i + 482) * 20;
      const progress = (phase + seconds / period) % 1;
      const x = width * (.04 + sceneNoise(this.seed, i + 483) * .92) + Math.sin(seconds * .3 + phase * 6) * 6;
      const y = height + 16 - progress * (height + 44), fade = Math.min(1, progress * 8, (1 - progress) * 8);
      glyph(i % 4 === 0 ? 3 : 2, i % 2, x, y, fade * (.14 + sceneNoise(this.seed, i + 484) * .09));
    }
    ctx.globalAlpha = 1;
  }
}
