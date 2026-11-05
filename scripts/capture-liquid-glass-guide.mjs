// Galería documental: frontend de producción + CLI/SQLite sintéticos, sin IPC nativo.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { startHeadlessBridge } from './headless-bridge.mjs';

const repo = fileURLToPath(new URL('../', import.meta.url));
process.chdir(repo);
const browser = process.env.NOTEKA_TEST_BROWSER || 'chromium';
if (!['chromium', 'webkit'].includes(browser)) throw Error('NOTEKA_TEST_BROWSER debe ser chromium o webkit');
if (!existsSync('dist/index.html') || !existsSync('target/debug/noteka')) {
  throw Error('Primero ejecutá npm run build y cargo build -p noteka-cli');
}
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const sources = ['src/style.css', 'src/lib/primary-color.ts', 'src/App.svelte',
  ...readdirSync('src/lib').filter(name => name.endsWith('.svelte') || name === 'sliding-tabs.ts').map(name => 'src/lib/' + name),
  'scripts/capture-liquid-glass-guide.mjs', 'scripts/capture-liquid-glass-guide.cjs'];
const sourceHashes = Object.fromEntries(sources.sort().map(path => [path, hash(path)]));
const output = resolve('output/playwright/liquid-glass-guide', browser);
mkdirSync(output, { recursive: true });
const bridge = await startHeadlessBridge();
const session = `noteka-design-${process.pid}`;
const config = join(bridge.root, 'playwright.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: {
  deviceScaleFactor: 2, serviceWorkers: 'block', reducedMotion: 'reduce',
} } }));
const run = args => new Promise((done, reject) => {
  const child = spawn('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', '-s=' + session, ...args], { stdio: 'inherit' });
  child.on('error', reject);
  child.on('exit', code => code === 0 ? done() : reject(Error('Playwright terminó con código ' + code)));
});
try {
  await run(['open', bridge.origin, '--config=' + config, ...(browser === 'webkit' ? ['--browser=webkit'] : [])]);
  await run(['run-code', '--filename=' + resolve('scripts/capture-liquid-glass-guide.cjs')]);
  const result = JSON.parse(readFileSync(join(bridge.root, 'results.json'), 'utf8')).at(-1);
  if (result.browser !== browser || result.errors.length || result.externalRequests) throw Error('Captura incompleta');
  for (const [path, digest] of Object.entries(sourceHashes)) {
    if (hash(path) !== digest) throw Error('La fuente cambió durante la captura: ' + path);
  }
  const destination = resolve('docs/design/liquid-glass', browser);
  mkdirSync(destination, { recursive: true });
  const captures = result.captures.map(capture => {
    const source = join(output, capture.file);
    cpSync(source, join(destination, capture.file));
    return { ...capture, sha256: hash(source), bytes: readFileSync(source).length };
  });
  const manifest = {
    generatedAt: new Date().toISOString(), browser, browserVersion: result.browserVersion,
    playwrightCli: '0.1.22', deviceScaleFactor: 2, reducedMotion: 'reduce',
    data: 'Workspace temporal nuevo, nombres ficticios, foto = icono local de Noteka. Sin datos de usuario.',
    scope: 'Frontend compilado con puente CLI Rust/SQLite. No acredita composición Tauri/macOS ni IPC nativo.',
    sourceHashes, captures,
    buildHashes: Object.fromEntries(['dist/index.html', ...readdirSync('dist/assets').map(name => 'dist/assets/' + name)].sort().map(path => [path, hash(path)])),
  };
  writeFileSync(join(destination, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
  console.log(`Galería ${browser}: ${captures.length} PNG en ${relative(repo, destination)}`);
} finally {
  try { await run(['close']); } finally { await bridge.close(); }
}
