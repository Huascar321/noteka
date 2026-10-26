export const COVER_SCENES = ['sharks', 'koi', 'jellyfish'] as const;
export type CoverScene = typeof COVER_SCENES[number];
export const SCENE_SECONDS = 42, SCENE_FADE_SECONDS = 1.4;
export function sceneNoise(seed: number, index: number): number {
  let value = Math.imul((seed + index * 7919) | 0, 1597334677);
  value = Math.imul(value ^ value >>> 15, 2246822519);
  return ((value ^ value >>> 13) >>> 0) / 4294967296;
}
export function coverSceneAt(seed: number, seconds: number) {
  const time = Math.max(0, Number.isFinite(seconds) ? seconds : 0);
  const step = Math.floor(time / SCENE_SECONDS), local = time - step * SCENE_SECONDS;
  const first = Math.floor(sceneNoise(seed, 0) * 3), direction = sceneNoise(seed, 1) > .5 ? 1 : 2;
  const current = COVER_SCENES[(first + step * direction) % 3], next = COVER_SCENES[(first + (step + 1) * direction) % 3];
  const t = Math.max(0, Math.min(1, (local - (SCENE_SECONDS - SCENE_FADE_SECONDS)) / SCENE_FADE_SECONDS));
  return { current, next, blend: t * t * (3 - 2 * t) };
}
export type KoiPose = { x: number; y: number; length: number; heading: number; phase: number; seed: number };
export function koiPoses(width: number, height: number, seconds: number, seed: number): KoiPose[] {
  return Array.from({ length: width < 600 ? 2 : 3 }, (_, i) => {
    const phase = sceneNoise(seed, i + 10) * Math.PI * 2 + seconds * (.1 + i * .019), direction = i % 2 ? -1 : 1;
    const rx = width * .09, ry = height * .07, theta = phase * direction;
    const dx = -rx * Math.sin(theta) * direction, dy = ry * Math.cos(theta) * direction;
    return { x: width * [.18,.80,.50][i] + rx * Math.cos(theta), y: height * .50 + ry * Math.sin(theta),
      length: Math.min(width * .23, height * .65, 220) * (.85 + sceneNoise(seed, i + 20) * .25),
      heading: Math.atan2(dy, dx), phase: seconds * (4.1 + i * .33) + phase, seed: seed + i * 193 };
  });
}
export function koiBody(u: number, length: number, phase: number) {
  return { x: length * (.47 - u * .84), y: Math.sin(phase - u * 4.7) * length * .045 * u * u,
    radius: length * (.018 + .115 * Math.sin(Math.PI * u) ** .85) * (.45 + .55 * Math.min(1, u / .09)) };
}
export type JellyPose = { x: number; y: number; radius: number; angle: number; pulse: number; phase: number; family: number; seed: number };
export function jellyPoses(width: number, height: number, seconds: number, seed: number): JellyPose[] {
  return Array.from({ length: width < 600 ? 3 : 4 }, (_, i) => {
    const phase = sceneNoise(seed, i + 41) * Math.PI * 2;
    return { x: width * [.16,.43,.72,.88][i] + Math.sin(seconds * .17 + phase) * width * .045,
      y: height * [.40,.55,.38,.66][i] + Math.sin(seconds * .22 + phase) * height * .09,
      radius: Math.min(width * .078, height * .20, 58) * (.82 + sceneNoise(seed, i + 45) * .3),
      angle: Math.sin(seconds * .3 + phase) * .18, pulse: .5 + .5 * Math.sin(seconds * 1.8 + phase),
      phase: seconds * 1.65 + phase, family: i % 2 ? 3 : 2, seed: seed + i * 317 };
  });
}
