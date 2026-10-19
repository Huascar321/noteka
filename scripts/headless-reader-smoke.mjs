// --isolated verifica el componente real sin modificar/importar App en aceptación.
import { spawn } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { startReaderBridge } from './headless-reader-bridge.mjs';
const isolated = process.argv.includes('--isolated');
const session = `noteka-reader-${process.pid}`;
function run(command, args) {
  return new Promise((done, reject) => {
    const child = spawn(command, args, { stdio: 'inherit' }); const timer = setTimeout(() => child.kill(), 1_200_000);
    child.on('error', e => { clearTimeout(timer); reject(e); }); child.on('close', code => { clearTimeout(timer); code ? reject(new Error(`${command}: ${code}`)) : done(); });
  });
}
const playwright = args => run('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args]);
let dist = resolve('dist');
if (isolated) {
  const root = mkdtempSync(join(process.env.NOTEKA_TEST_TMP || tmpdir(), 'reader-isolated-build-')); dist = join(root, 'dist');
  mkdirSync(join(root, 'entry')); const entry = join(root, 'entry');
  writeFileSync(join(entry, 'index.html'), `<html><head><meta charset="UTF-8"/><link rel="icon" href="data:,"/></head><body><main id="reader-mount"></main><script type="module">import {mount} from ${JSON.stringify(resolve('node_modules/svelte/src/index-client.js'))};import Reader from ${JSON.stringify(resolve('src/lib/AttachmentGenerationPanel.svelte'))};import ${JSON.stringify(resolve('src/style.css'))};document.documentElement.dataset.theme=new URL(location).searchParams.get('theme')||'dark';mount(Reader,{target:document.getElementById('reader-mount'),props:{workspace:new URL(location).searchParams.get('workspace')}});</script></body></html>`);
  const { build } = await import('vite'), { svelte } = await import('@sveltejs/vite-plugin-svelte');
  await build({ configFile: false, root: entry, publicDir: false, plugins: [svelte()], build: { outDir: dist } });
} else await run('npm', ['run', 'build']);
const bridge = await startReaderBridge(dist, isolated); console.log(JSON.stringify({ reader_root: bridge.root, isolated }));
const config = join(bridge.root, 'playwright.config.json'); writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block' } } }));
try {
  await playwright(['open', bridge.origin, `--config=${config}`]);
  await playwright(['run-code', `--filename=${resolve('tests/headless-reader.cjs')}`]);
  const reports = ['dark', 'light'].map(theme => JSON.parse(readFileSync(join(bridge.root, `reader-${theme}-results.json`), 'utf8')));
  if (reports.some(r => !r.ok || r.external_requests)) throw new Error('Lectura headless incompleta');
  console.log(JSON.stringify({ root: bridge.root, isolated, checks: reports.reduce((n, r) => n + r.checks.length, 0), external_requests: 0 }));
} finally { try { await playwright(['close']); } finally { await bridge.close(); } }
