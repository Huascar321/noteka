// Production artwork and native assets. No model, network or personal workspace.
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
process.chdir(root);
const check = process.argv.includes('--check');
const geometryPath = 'src/lib/noteka-brand.json';
const document = 'src-tauri/icons/Noteka.icon';
const manifestPath = 'src-tauri/icons/brand-manifest.json';
const mark = JSON.parse(readFileSync(geometryPath, 'utf8'));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const checkPng = (bytes, size, name) => {
  if (bytes.length < 26 || bytes.readUInt32BE(16) !== size || bytes.readUInt32BE(20) !== size || bytes[24] !== 8 || bytes[25] !== 6) {
    throw Error(`${name} must be ${size}×${size}, 8-bit RGBA PNG.`);
  }
};
const layer = (path, color) => `<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024"><g transform="translate(128 122) scale(3)"><path fill="${color}" d="${path}"/></g></svg>\n`;
const textOutputs = {
  [`${document}/Assets/note.svg`]: layer(mark.body, mark.blue),
  [`${document}/Assets/peel.svg`]: layer(mark.peel, mark.paper),
  'public/noteka.svg': `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="${mark.viewBox}">
<style>.tile{fill:#f7f7f5}.note{fill:${mark.blue}}@media(prefers-color-scheme:dark){.tile{fill:#202020}.note{fill:${mark.blueDark}}}</style>
<rect class="tile" width="256" height="256" rx="56"/>
<g transform="translate(22 18) scale(.84)"><path class="note" d="${mark.body}"/><path fill="${mark.paper}" d="${mark.peel}"/></g>
</svg>\n`,
};

if (check) {
  for (const [path, expected] of Object.entries(textOutputs)) {
    if (!existsSync(path) || readFileSync(path, 'utf8') !== expected) throw Error(`Brand asset is stale: ${path}. Run npm run brand:generate on macOS.`);
  }
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  const sourcePaths = [geometryPath, `${document}/icon.json`, 'scripts/generate-brand.mjs', 'scripts/normalize-icon-png.swift'];
  const outputPaths = [...Object.keys(textOutputs), 'src-tauri/icons/32x32.png', 'src-tauri/icons/128x128.png', 'src-tauri/icons/256x256.png', 'src-tauri/icons/icon.icns', 'src-tauri/icons/icon.ico', 'src-tauri/icons/Assets.car'];
  for (const [entries, paths] of [[manifest.sources, sourcePaths], [manifest.outputs, outputPaths]]) {
    for (const path of paths) {
      if (!existsSync(path) || entries[path] !== hash(path)) throw Error(`Brand integrity mismatch: ${path}. Run npm run brand:generate on macOS.`);
    }
  }
  for (const size of [32, 128, 256]) checkPng(readFileSync(`src-tauri/icons/${size}x${size}.png`), size, `PNG ${size}`);
  const ico = readFileSync('src-tauri/icons/icon.ico');
  if (ico.readUInt16LE(2) !== 1 || ico.readUInt16LE(4) !== 6) throw Error('ICO must contain six icon frames.');
  const sizes = new Set();
  for (let index = 0; index < 6; index++) {
    const entry = 6 + index * 16;
    const size = ico[entry] || 256;
    const length = ico.readUInt32LE(entry + 8), offset = ico.readUInt32LE(entry + 12);
    checkPng(ico.subarray(offset, offset + length), size, `ICO ${size}`); sizes.add(size);
  }
  if (![16, 24, 32, 48, 64, 256].every(size => sizes.has(size))) throw Error('ICO frame sizes are incomplete.');
  console.log(`Brand: ${sourcePaths.length} sources and ${outputPaths.length} outputs verified.`);
} else {
  if (process.platform !== 'darwin') throw Error('Native icon generation requires macOS with Xcode/Icon Composer. Checked-in resources support ordinary builds elsewhere.');
  const developer = execFileSync('xcode-select', ['-p'], { encoding: 'utf8' }).trim();
  const ictool = resolve(developer, '../Applications/Icon Composer.app/Contents/Executables/ictool');
  if (!existsSync(ictool)) throw Error('Icon Composer is required in the selected Xcode installation.');
  for (const [path, content] of Object.entries(textOutputs)) {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, content);
  }
  const temporary = mkdtempSync(join(tmpdir(), 'noteka-brand-'));
  try {
    const compiled = join(temporary, 'compiled');
    mkdirSync(compiled);
    execFileSync('xcrun', ['actool', document, '--compile', compiled, '--platform', 'macosx', '--target-device', 'mac', '--minimum-deployment-target', '11.0', '--app-icon', 'Noteka', '--include-all-app-icons', '--output-partial-info-plist', join(compiled, 'partial.plist')], { stdio: 'inherit' });
    copyFileSync(join(compiled, 'Assets.car'), 'src-tauri/icons/Assets.car');
    copyFileSync(join(compiled, 'Noteka.icns'), 'src-tauri/icons/icon.icns');
    const render = (rendition, size, path) => execFileSync(ictool, [document, '--export-image', '--output-file', path, '--platform', 'macOS', '--rendition', rendition, '--width', String(size), '--height', String(size), '--scale', '1'], { stdio: 'inherit' });
    for (const size of [32, 128, 256]) render('Default', size, `src-tauri/icons/${size}x${size}.png`);
    // Standard multi-resolution ICO with PNG-encoded frames; no image library.
    const framePaths = [32, 16, 24, 48, 64, 256].map(size => {
      const path = join(temporary, `${size}.png`);
      render('Default', size, path);
      return { size, path };
    });
    execFileSync('xcrun', ['swift', 'scripts/normalize-icon-png.swift', ...[32, 128, 256].map(size => `src-tauri/icons/${size}x${size}.png`), ...framePaths.map(frame => frame.path)], { stdio: 'inherit' });
    const frames = framePaths.map(({ size, path }) => ({ size, bytes: readFileSync(path) }));
    const header = Buffer.alloc(6 + frames.length * 16);
    header.writeUInt16LE(1, 2); header.writeUInt16LE(frames.length, 4);
    let offset = header.length;
    frames.forEach(({ size, bytes }, index) => {
      const entry = 6 + index * 16;
      header[entry] = size === 256 ? 0 : size; header[entry + 1] = header[entry];
      header.writeUInt16LE(1, entry + 4); header.writeUInt16LE(32, entry + 6);
      header.writeUInt32LE(bytes.length, entry + 8); header.writeUInt32LE(offset, entry + 12);
      offset += bytes.length;
    });
    writeFileSync('src-tauri/icons/icon.ico', Buffer.concat([header, ...frames.map(frame => frame.bytes)]));
    const preview = resolve('output/playwright/brand/native');
    mkdirSync(preview, { recursive: true });
    for (const rendition of ['Default', 'Dark', 'ClearLight', 'ClearDark']) render(rendition, 512, join(preview, `${rendition}.png`));
    const sources = Object.fromEntries([geometryPath, `${document}/icon.json`, 'scripts/generate-brand.mjs', 'scripts/normalize-icon-png.swift'].map(path => [path, hash(path)]));
    const outputs = Object.fromEntries([...Object.keys(textOutputs), 'src-tauri/icons/32x32.png', 'src-tauri/icons/128x128.png', 'src-tauri/icons/256x256.png', 'src-tauri/icons/icon.icns', 'src-tauri/icons/icon.ico', 'src-tauri/icons/Assets.car'].map(path => [path, hash(path)]));
    writeFileSync(manifestPath, JSON.stringify({ generator: 'scripts/generate-brand.mjs', xcode: execFileSync('xcodebuild', ['-version'], { encoding: 'utf8' }).trim(), sources, outputs }, null, 2) + '\n');
    console.log(`Generated native icon, compatibility assets and previews in ${preview}`);
  } finally { rmSync(temporary, { recursive: true, force: true }); }
}
