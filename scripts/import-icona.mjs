// Explicit development-only import. Normal generation/build never reads upstream or uses a network.
// Usage (Node >=22.22): node scripts/import-icona.mjs /path/to/pinned/Icona
import { registerHooks } from 'node:module';
import { execFileSync } from 'node:child_process';
import { mkdir, readFile, realpath, writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { ICONA_REVISION, ICON_SELECTION, SIDEBAR_MOTIONS, SIDEBAR_MOTION_DURATION_MS } from './icon-catalog.mjs';

if (!process.argv[2]) throw new Error('Pass a local Icona checkout; see third-party/icona/README.md.');
const root = await realpath(process.argv[2]);
const revision = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
if (revision !== ICONA_REVISION) throw new Error(`Expected Icona ${ICONA_REVISION}, got ${revision}.`);
const changes = execFileSync('git', ['status', '--porcelain', '--untracked-files=all', '--', 'components', 'packages/icons/package.json'], { cwd: root, encoding: 'utf8' }).trim();
if (changes) throw new Error('Use a clean upstream source tree, including untracked files.');
// Upstream TypeScript uses extensionless relative imports. Native Node strips its types.
registerHooks({ resolve(specifier, context, nextResolve) {
  try { return nextResolve(specifier, context); } catch (error) {
    if (specifier.startsWith('.') && !/\.[a-z]+$/i.test(specifier)) return nextResolve(`${specifier}.ts`, context);
    throw error;
  }
} });
const upstream = path => import(pathToFileURL(`${root}/${path}`).href);
const { iconVariantsByKey } = await upstream('components/icons/iconLibrary.ts');
const { staticSvg } = await upstream('components/export/static.ts');
const { generateCode } = await upstream('components/export/code.ts');
const { DEFAULT_OPTIONS } = await upstream('components/export/options.ts');
const assets = {};
for (const key of [...new Set([...Object.values(ICON_SELECTION), 'plus-outline'])].sort()) {
  const variant = iconVariantsByKey.get(key);
  if (!variant) throw new Error(`Missing upstream variant ${key}`);
  const tree = staticSvg(variant, 24, { strokeWidth: variant.style === 'fill' ? 1.5 : 2.4 });
  delete tree.attrs.width; delete tree.attrs.height; delete tree.attrs.xmlns;
  assets[key] = tree;
}
const motions = {};
// Reuse the exact Tasks export even where upstream's per-icon menu omits Jelly.
// Jelly transforms the whole .fx group; its motion is independent of the geometry.
const reference = iconVariantsByKey.get(ICON_SELECTION.task);
const options = { ...DEFAULT_OPTIONS, strokeWidth: assets[reference.key].attrs.strokeWidth, motionId: 'jelly' };
const base = generateCode(reference, options, 'svg').scene;
if (base.info.motion?.id !== 'jelly' || !base.css || base.info.pixel || base.info.tier !== 'animated') throw new Error('No Tasks Jelly CSS motion');
const scene = generateCode(reference, { ...options, speed: base.info.loopSeconds / (SIDEBAR_MOTION_DURATION_MS / 1000) }, 'svg').scene;
if (scene.body.length !== 1 || scene.body[0].tag !== 'g' || scene.body[0].attrs.class !== 'fx') throw new Error('Review Jelly group structure');
for (const [name, motionId] of Object.entries(SIDEBAR_MOTIONS)) {
  const variant = iconVariantsByKey.get(ICON_SELECTION[name]);
  if (motionId !== 'jelly') throw new Error(`Expected shared Jelly motion for ${name}`);
  const body = [{ ...scene.body[0], children: assets[variant.key].children }];
  if (name === 'task' && JSON.stringify(body) !== JSON.stringify(scene.body)) throw new Error('Tasks Jelly geometry changed');
  motions[name] = {
    source: variant.key, motionId, scope: scene.scope, viewBox: scene.viewBox,
    durationMs: SIDEBAR_MOTION_DURATION_MS, css: scene.css, body,
  };
}
const destination = new URL('../third-party/icona/', import.meta.url);
await mkdir(destination, { recursive: true });
await writeFile(new URL('assets.json', destination), JSON.stringify({ revision, assets, motions }, null, 2) + '\n');
await writeFile(new URL('upstream-package.json', destination), await readFile(`${root}/packages/icons/package.json`));
console.log(`Imported ${Object.keys(assets).length} SVG variants and ${Object.keys(motions).length} CSS scenes at ${revision}.`);
