// Focused reference gallery: real compiled UI + isolated CLI/SQLite corpus.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';

if (!existsSync('dist/index.html') || !existsSync('target/debug/noteka')) throw Error('Run npm run build and cargo build -p noteka-cli first.');
const browser = process.env.NOTEKA_TEST_BROWSER || 'chromium';
if (!['chromium', 'webkit'].includes(browser)) throw Error('Unsupported browser.');
const sources = ['src/App.svelte', 'src/style.css', 'src/lib/NotekaMark.svelte', 'src/lib/noteka-brand.json', 'public/noteka.svg', 'scripts/capture-brand.mjs', 'scripts/capture-brand.cjs'];
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const sourceHashes = Object.fromEntries(sources.map(path => [path, hash(path)]));
const bridge = await startHeadlessBridge();
const session = `noteka-brand-${process.pid}`;
const config = join(bridge.root, 'playwright.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { deviceScaleFactor: 2, serviceWorkers: 'block', reducedMotion: 'reduce' } } }));
const run = args => new Promise((done, reject) => {
  const child = spawn('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', '-s=' + session, ...args], { stdio: 'inherit' });
  child.on('error', reject); child.on('exit', code => code === 0 ? done() : reject(Error(`Playwright: ${code}`)));
});
const output = resolve('output/playwright/brand', browser);
mkdirSync(output, { recursive: true });
try {
  await run(['open', bridge.origin, '--config=' + config, ...(browser === 'webkit' ? ['--browser=webkit'] : [])]);
  await run(['run-code', '--filename=' + resolve('scripts/capture-brand.cjs')]);
  const result = JSON.parse(readFileSync(join(bridge.root, 'results.json'), 'utf8')).at(-1);
  if (!result?.ok || result.external.length || result.errors.length) throw Error('Brand capture incomplete.');
  for (const path of sources) if (hash(path) !== sourceHashes[path]) throw Error(`Source changed during capture: ${path}`);
  const destination = resolve('docs/design/brand', browser);
  mkdirSync(destination, { recursive: true });
  for (const name of result.captures) copyFileSync(join(output, name), join(destination, name));
  const captures = result.captures.map(name => ({ file: name, sha256: hash(join(destination, name)) }));
  writeFileSync(join(destination, 'manifest.json'), JSON.stringify({ browser, browserVersion: result.browserVersion, sourceHashes, captures, checks: result.checks, scope: 'Compiled frontend + real CLI/SQLite synthetic workspace; no native screen or personal data.' }, null, 2) + '\n');
  console.log(`Brand gallery: ${captures.length} captures, ${result.checks.length} checks (${browser}).`);
} finally { try { await run(['close']); } finally { await bridge.close(); } }
