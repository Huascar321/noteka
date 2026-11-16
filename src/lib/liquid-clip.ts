/** Prepare the reveal once; playback must not measure layout on every frame. */
export type ClipBox = { left: number; top: number; width: number; height: number };
type Shape = { offset: number; x: number; y: number; sx: number; sy: number; rx: number; ry: number; easing: string };

function eased(time: number, easing: string): number {
  const values = easing === 'ease-in-out' ? [.42, 0, .58, 1] : easing === 'ease-out' ? [0, 0, .58, 1]
    : easing.match(/^cubic-bezier\(([^)]+)\)$/)?.[1].split(',').map(Number);
  if (!values) return time;
  const [x1, y1, x2, y2] = values;
  const curve = (t: number, a: number, b: number) => 3 * (1 - t) ** 2 * t * a + 3 * (1 - t) * t * t * b + t ** 3;
  let lo = 0, hi = 1;
  for (let i = 0; i < 18; i++) { const middle = (lo + hi) / 2; if (curve(middle, x1, x2) < time) lo = middle; else hi = middle; }
  return curve((lo + hi) / 2, y1, y2);
}

export function materialShapes(frames: Keyframe[], radius: string): Shape[] {
  return frames.map((frame, index) => {
    const matrix = new DOMMatrixReadOnly(String(frame.transform ?? 'none'));
    const radii = String(frame.borderRadius ?? radius).split(/[\s/]+/).map(Number.parseFloat);
    return { offset: frame.offset ?? index / (frames.length - 1), x: matrix.e, y: matrix.f, sx: matrix.a, sy: matrix.d,
      rx: radii[0] || 0, ry: radii[1] ?? radii[0] ?? 0, easing: frame.easing ?? 'linear' };
  });
}

export function clippingFrames(shapes: Shape[], surface: ClipBox, content: ClipBox): Keyframe[] {
  const result: Keyframe[] = [];
  for (let i = 0; i < shapes.length - 1; i++) {
    const a = shapes[i], b = shapes[i + 1];
    for (let step = i ? 1 : 0; step <= 32; step++) {
      const time = step / 32, progress = step === 0 ? 0 : step === 32 ? 1 : eased(time, a.easing);
      const mix = (key: 'x' | 'y' | 'sx' | 'sy' | 'rx' | 'ry') => a[key] + (b[key] - a[key]) * progress;
      const left = surface.left + mix('x'), top = surface.top + mix('y');
      const right = left + surface.width * mix('sx'), bottom = top + surface.height * mix('sy');
      result.push({ offset: a.offset + (b.offset - a.offset) * time,
        clipPath: `inset(${top - content.top}px ${content.left + content.width - right}px ${content.top + content.height - bottom}px ${left - content.left}px round ${Math.abs(mix('rx') * mix('sx'))}px / ${Math.abs(mix('ry') * mix('sy'))}px)` });
    }
  }
  return result;
}
