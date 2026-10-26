import { AsciiOceanRenderer } from './ascii-ocean-renderer';
import { AsciiCreaturesRenderer } from './ascii-creatures';
import { mixedOceanActors } from './mixed-ocean-scene';

export class AsciiCoverRenderer {
  private ctx: CanvasRenderingContext2D;
  private sharks = new AsciiOceanRenderer(document.createElement('canvas'));
  private koi: AsciiCreaturesRenderer;
  private jellyfish: AsciiCreaturesRenderer;
  private width = 0;
  private height = 0;
  private dpr = 1;
  private seconds = 0;

  constructor(private canvas: HTMLCanvasElement, private seed = Math.floor(Math.random() * 2147483647)) {
    const ctx = canvas.getContext('2d');
    if (!ctx) throw Error('Canvas2D unavailable');
    this.ctx = ctx;
    this.koi = new AsciiCreaturesRenderer(document.createElement('canvas'), 'koi', seed);
    this.jellyfish = new AsciiCreaturesRenderer(document.createElement('canvas'), 'jellyfish', seed);
  }

  resize(width: number, height: number, dpr: number, dark: boolean, ink: string) {
    this.width = Math.max(0, width); this.height = Math.max(0, height);
    this.dpr = Math.min(2, Math.max(1, dpr));
    this.canvas.width = Math.round(this.width * this.dpr);
    this.canvas.height = Math.round(this.height * this.dpr);
    this.sharks.resize(this.width, this.height, dpr, dark, ink, false);
    this.koi.resize(this.width, this.height, dpr, dark);
    this.jellyfish.resize(this.width, this.height, dpr, dark);
    this.draw(this.seconds, 0);
  }

  draw(seconds: number, delta: number) {
    this.seconds = seconds;
    if (!this.width || !this.height) return;
    const actors = mixedOceanActors(this.width, this.height, seconds, this.seed), ctx = this.ctx;
    ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    ctx.clearRect(0, 0, this.width, this.height);
    this.sharks.drawEnvironment(ctx, seconds);
    for (const actor of actors) {
      if (actor.family === 'ocean') this.sharks.drawActor(ctx, actor.index, actor.pose, delta);
      else this[actor.family].drawActor(ctx, actor.pose, actor.opacity);
    }
    ctx.globalAlpha = 1;
    this.canvas.dataset.scene = 'mixed';
    this.canvas.dataset.actorCount = String(actors.length);
    delete this.canvas.dataset.sceneBlend;
  }
}
