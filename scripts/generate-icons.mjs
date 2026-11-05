import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { ICONA_REVISION, ICON_SELECTION, ICON_ADAPTATIONS, SIDEBAR_MOTIONS, SIDEBAR_MOTION_DURATION_MS } from './icon-catalog.mjs';

const check = process.argv.includes('--check');
const data = JSON.parse(await readFile(new URL('../third-party/icona/assets.json', import.meta.url), 'utf8'));
if (data.revision !== ICONA_REVISION) throw new Error('Review the Icona revision, catalog and notices together.');
const SCOPE = '__NOTEKA_ICON_SCOPE__';
const tags = new Set(['path', 'circle', 'line', 'rect', 'g', 'defs', 'mask']);
const allowedAttrs = new Set(['viewBox', 'fill', 'stroke', 'strokeWidth', 'strokeLinecap', 'strokeLinejoin', 'd', 'cx', 'cy', 'r', 'x', 'y', 'x1', 'y1', 'x2', 'y2', 'width', 'height', 'id', 'maskUnits', 'mask', 'class', 'strokeDasharray', 'strokeDashoffset', 'strokeOpacity', 'fillOpacity', 'transform']);
const escape = value => String(value).replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;');
function attributes(node) {
  return Object.fromEntries(Object.entries(node.attrs ?? {}).map(([key, raw]) => {
    if (!allowedAttrs.has(key) || !['number', 'string'].includes(typeof raw)) throw new Error(`Invalid SVG attribute: ${key}`);
    let value = String(raw);
    if (/[<>]|(?:javascript:|data:|https?:|\/\/)/i.test(value)) throw new Error(`Non-local SVG value: ${key}`);
    if (key === 'id') value = `${SCOPE}-${value}`;
    value = value.replace(/url\(#([a-zA-Z0-9_-]+)\)/g, `url(#${SCOPE}-$1)`);
    if (value.includes('url(') && !/^url\(#[a-zA-Z0-9_-]+\)$/.test(value)) throw new Error('Non-local SVG reference');
    const name = key === 'viewBox' || key === 'maskUnits' ? key : key.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`);
    return [name, value];
  }));
}
function element(node) {
  if (!tags.has(node.tag)) throw new Error(`Invalid SVG tag: ${node.tag}`);
  return `<${node.tag}${Object.entries(attributes(node)).map(([key, value]) => ` ${key}="${escape(value)}"`).join('')}>${(node.children ?? []).map(element).join('')}</${node.tag}>`;
}
const group = (transform, children, attrs = {}) => ({ tag: 'g', attrs: { transform, ...attrs }, children });
const walk = node => [node, ...(node.children ?? []).flatMap(walk)];
function adaptedTree(key, source) {
  const tree = structuredClone(data.assets[source]);
  if (!tree || tree.tag !== 'svg') throw new Error(`Missing SVG: ${source}`);
  switch (ICON_ADAPTATIONS[key]) {
    case 'mirror': tree.children = [group('translate(24 0) scale(-1 1)', tree.children)]; break;
    case 'open-shackle': {
      // Rotate the official shackle geometry around its left hinge.
      const shackle = walk(tree).find(node => node.tag === 'path' && node.attrs.d === 'M8 10.5V7.5A4 4 0 0 1 16 7.5V10.5');
      if (!shackle) throw new Error('Review open-lock adaptation');
      shackle.attrs.transform = 'rotate(-35 8 10.5)';
      break;
    }
    case 'user-plus': tree.children = [
      group('translate(-1 3) scale(.8)', tree.children),
      group('translate(14 1) scale(.42)', structuredClone(data.assets['plus-outline'].children), { strokeWidth: 3.5 }),
    ]; break;
    case 'users': tree.children = [
      group('translate(8 0) scale(.65)', structuredClone(tree.children)),
      group('translate(-1 5) scale(.8)', tree.children),
    ]; break;
    case 'double-arrow': tree.children = [
      group('translate(-2 0) scale(.8 1)', structuredClone(tree.children)),
      group('translate(7 0) scale(.8 1)', tree.children),
    ]; break;
  }
  return tree;
}
const assets = [], indices = new Map(), keys = {};
for (const [key, source] of Object.entries(ICON_SELECTION)) {
  const tree = adaptedTree(key, source);
  const asset = { attrs: attributes(tree), body: tree.children.map(element).join('') };
  const signature = JSON.stringify(asset);
  if (!indices.has(signature)) { indices.set(signature, assets.length); assets.push(asset); }
  keys[key] = indices.get(signature);
}
const motions = {};
for (const [key, motionId] of Object.entries(SIDEBAR_MOTIONS)) {
  const motion = data.motions[key];
  if (motion.source !== ICON_SELECTION[key] || motion.motionId !== motionId || motion.viewBox !== '0 0 24 24'
    || motion.durationMs !== SIDEBAR_MOTION_DURATION_MS) throw new Error(`Review sidebar motion ${key}`);
  // Keep official keyframes; replace export's hover/loop triggers with one explicit cycle.
  let css = motion.css
    .replaceAll('[data-animate="hover"]:is(:hover,:focus-visible)', '[data-icon-playing="true"]')
    .replaceAll('[data-animate="loop"]', '[data-icon-playing="true"]')
    .replace(/var\(--icona-iter,1\)( alternate)?/g, (_, alternate) => alternate ? '2 alternate' : '1')
    .replaceAll('infinite', '1')
    .replaceAll(motion.scope, `${SCOPE}-${motion.scope}`);
  if (/@import|url\(|[<>]|:hover|infinite|data-animate|javascript:/i.test(css)) throw new Error(`Invalid motion CSS: ${key}`);
  motions[key] = { scope: `${SCOPE}-${motion.scope}`, durationMs: motion.durationMs, body: motion.body.map(element).join(''), css };
}
const header = `// Generated by scripts/generate-icons.mjs from Icona ${ICONA_REVISION}.\n// Local SVG/CSS assets; provenance and MIT notice: third-party/icona/. Do not edit.\n`;
const staticSource = header + 'export type StaticIcon = { attrs: Record<string, string>; body: string };\nconst ASSETS: StaticIcon[] = [\n'
  + assets.map(asset => `  ${JSON.stringify(asset)},`).join('\n') + '\n];\nexport const ICONS = {\n'
  + Object.entries(keys).map(([key, index]) => `  ${JSON.stringify(key)}: ASSETS[${index}],`).join('\n') + '\n};\n';
const styles = [...new Set(Object.values(motions).map(motion => motion.css))];
const navSource = header + 'export type SidebarIconMotion = { scope: string; durationMs: number; body: string; css: string };\n'
  + styles.map((css, index) => `const CSS_${index} = ${JSON.stringify(css)};\n`).join('')
  + 'export const SIDEBAR_ICONS: Record<string, SidebarIconMotion> = {\n'
  + Object.entries(motions).map(([key, { css, ...motion }]) => `  ${JSON.stringify(key)}: {${JSON.stringify(motion).slice(1, -1)},"css":CSS_${styles.indexOf(css)}},`).join('\n') + '\n};\n';
const outputs = [['../src/lib/icons.generated.ts', staticSource], ['../src/lib/sidebar-icons.generated.ts', navSource]];
for (const name of ['LICENSE.txt', 'NOTICE.txt']) {
  outputs.push([`../public/licenses/icona/${name}`, await readFile(new URL(`../third-party/icona/${name}`, import.meta.url), 'utf8')]);
}
for (const [path, source] of outputs) {
  const url = new URL(path, import.meta.url);
  if (check) {
    if (await readFile(url, 'utf8') !== source) throw new Error(`Stale icons: npm run icons:generate (${path})`);
  } else {
    await mkdir(dirname(fileURLToPath(url)), { recursive: true }); await writeFile(url, source);
  }
}
console.log(`${check ? 'Checked' : 'Generated'} ${Object.keys(keys).length} Icona icons (${assets.length} unique SVGs), ${Object.keys(motions).length} lazy CSS scenes.`);
