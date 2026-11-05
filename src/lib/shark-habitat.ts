type GlyphPainter = { draw(ctx: CanvasRenderingContext2D, index: number, x: number, y: number, alpha: number): void };
type LightCell = { x: number; y: number; phase: number; band: number; opacity: number; glyph: number };
type RockCell = { x: number; y: number; opacity: number; glyph: number };
function noise(seed: number) { const n = Math.sin(seed * 127.1 + 17.7) * 43758.5453; return n - Math.floor(n); }

export class SharkHabitatRenderer {
  private width = 0;
  private height = 0;
  private light: LightCell[] = [];
  private rocks: RockCell[] = [];

  resize(width: number, height: number) {
    if (width === this.width && height === this.height) return;
    this.width = width; this.height = height; this.light = []; this.rocks = [];
    if (!width || !height) return;
    // Cache three sparse ribbons; only their gentle displacement changes in time.
    for (let band = 0; band < 3; band++) for (let x = 6; x < width; x += 12) {
      const u = x / width, envelope = Math.sin(Math.PI * u) ** 2;
      const phase = u * 8 + band * 2.1;
      for (let row = -1; row <= 1; row++) {
        if (noise(x + band * 97 + row * 31) > .68) continue;
        this.light.push({ x, y: height * [.12, .48, .82][band] + Math.sin(phase) * 9 + row * 8,
          phase, band, opacity: envelope * (row === 0 ? .12 : .045), glyph: row === 0 ? 1 : 0 });
      }
    }
    // One low, jagged outcrop, cropped by the lower-right edge of the cover.
    const extent = Math.min(width * .23, 230), rise = Math.min(height * .24, 70);
    const ridge = [[0, .08], [.15, .25], [.28, .64], [.46, .52], [.62, .90], [.82, .72], [1, .82]];
    for (let x = Math.floor((width - extent) / 6) * 6; x < width; x += 6) {
      const u = Math.max(0, (x - width + extent) / extent);
      let segment = 1; while (segment < ridge.length - 1 && u > ridge[segment][0]) segment++;
      const a = ridge[segment - 1], b = ridge[segment], fraction = (u - a[0]) / (b[0] - a[0]);
      const top = height - rise * (a[1] + (b[1] - a[1]) * fraction);
      const first = Math.ceil(top / 8) * 8;
      for (let y = first; y < height; y += 8) {
        const texture = noise(x * 3 + y * 17), edge = Math.min(1, u * 6);
        if (y > first && texture > .78) continue;
        this.rocks.push({ x, y, opacity: edge * (y === first ? .22 : .10 + texture * .10),
          glyph: y === first ? 13 : [0, 1, 3, 6, 8][Math.floor(texture * 5)] });
      }
    }
  }

  draw(ctx: CanvasRenderingContext2D, seconds: number, atlas: GlyphPainter) {
    for (const cell of this.light) {
      const drift = Math.sin(seconds * .13 + cell.phase) * 6 + Math.sin(seconds * .065 + cell.phase * 1.6) * 3;
      const glow = .78 + Math.sin(seconds * .19 + cell.phase * .16 + cell.band) * .22;
      atlas.draw(ctx, cell.glyph, cell.x, cell.y + drift, cell.opacity * glow);
    }
    for (const cell of this.rocks) atlas.draw(ctx, cell.glyph, cell.x, cell.y, cell.opacity);
    ctx.globalAlpha = 1;
  }
}
