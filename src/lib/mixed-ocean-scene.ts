import { oceanPoses, type SharkPose } from './shark-scene.ts';
import { sceneNoise, type KoiPose, type JellyPose } from './ascii-cover-scene.ts';

type ActorBase = { id: string; depth: number; opacity: number };
export type MixedOceanActor =
  | (ActorBase & { family: 'ocean'; index: number; pose: SharkPose })
  | (ActorBase & { family: 'koi'; pose: KoiPose })
  | (ActorBase & { family: 'jellyfish'; pose: JellyPose });

// Shared depth means a distant koi can pass behind a shark and a near jellyfish
// in front of both. Smaller geometry plus lower opacity supplies depth without blur.
export function depthAppearance(depth: number) {
  const bounded = Math.max(0, Math.min(1, depth));
  return { scale: 1.1 - bounded * .65, opacity: .94 - bounded * .69 };
}

export function mixedOceanActors(width: number, height: number, seconds: number, seed: number): MixedOceanActor[] {
  if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) return [];
  const time = Math.max(0, Number.isFinite(seconds) ? seconds : 0), narrow = width < 600;
  const actors: MixedOceanActor[] = oceanPoses(width, height, time).map((pose, index) => {
    const depth = [.29, .67, .89, .57][index] + Math.sin(time * .09 + index) * .025;
    const look = depthAppearance(depth);
    return { family: 'ocean', id: 'ocean-' + index, index, depth, opacity: look.opacity,
      pose: { ...pose, size: pose.size * .85 * look.scale, opacity: look.opacity, depth } };
  });
  for (let i = 0; i < (narrow ? 2 : 3); i++) {
    const initial = sceneNoise(seed, i + 10) * Math.PI * 2, direction = i % 2 ? -1 : 1;
    const phase = initial + time * (.09 + i * .017) * direction, rx = width * .065, ry = height * .09;
    const depth = [.10, .87, .54][i] + Math.sin(time * .08 + initial) * .025;
    const look = depthAppearance(depth);
    actors.push({ family: 'koi', id: 'koi-' + i, depth, opacity: look.opacity, pose: {
      x: width * [.22, .83, .54][i] + rx * Math.cos(phase),
      y: height * [.30, .27, .72][i] + ry * Math.sin(phase),
      length: Math.min(width * .21, height * .66, 210) * look.scale,
      heading: Math.atan2(ry * Math.cos(phase) * direction, -rx * Math.sin(phase) * direction),
      phase: time * (4.1 + i * .33) + initial, seed: seed + i * 193
    } });
  }
  for (let i = 0; i < (narrow ? 3 : 4); i++) {
    const initial = sceneNoise(seed, i + 41) * Math.PI * 2;
    const depth = [.18, .46, .76, .94][i] + Math.sin(time * .07 + initial) * .018;
    const look = depthAppearance(depth);
    actors.push({ family: 'jellyfish', id: 'jelly-' + i, depth, opacity: look.opacity, pose: {
      x: width * [.13, .63, .91, .37][i] + Math.sin(time * .14 + initial) * width * .025,
      y: height * [.53, .20, .51, .22][i] + Math.sin(time * .19 + initial) * height * .045,
      radius: Math.min(width * .065, height * .155, 48) * look.scale,
      angle: Math.sin(time * .26 + initial) * .18, pulse: .5 + .5 * Math.sin(time * 1.8 + initial),
      phase: time * 1.65 + initial, family: i % 2 ? 3 : 2, seed: seed + i * 317
    } });
  }
  return actors.sort((a, b) => b.depth - a.depth || a.id.localeCompare(b.id));
}
