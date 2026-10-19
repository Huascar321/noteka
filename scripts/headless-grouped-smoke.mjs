// Recorrido focalizado; la regresión de 606 checks sigue en test:headless.
import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';
const session = `noteka-grouped-${process.pid}`;
function run(command, args) {
  return new Promise((done, reject) => {
    const child = spawn(command, args, { stdio: 'inherit' });
    const timer = setTimeout(() => child.kill(), 1_200_000);
    child.on('error', error => { clearTimeout(timer); reject(error); });
    child.on('close', (code, signal) => { clearTimeout(timer); if (code === 0) done(); else reject(new Error(`${command}: code=${code}, signal=${signal}`)); });
  });
}
const playwright = args => run('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args]);
await run('npm', ['run', 'build']);
const bridge = await startHeadlessBridge();
const config = join(bridge.root, 'playwright.config.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block' } } }));
try {
  await playwright(['open', bridge.origin, `--config=${config}`]);
  await playwright(['run-code', `--filename=${resolve('tests/headless-grouped.cjs')}`]);
  const reports = ['dark', 'light'].map(theme => JSON.parse(readFileSync(join(bridge.root, `evidence-${theme}-results.json`), 'utf8')));
  if (reports.some(r => !r.ok || r.external_requests)) throw new Error('Evidencia headless incompleta/fallida');
  console.log(JSON.stringify({ root: bridge.root, checks: reports.reduce((n, r) => n + r.checks.length, 0), themes: reports.map(r => r.theme), external_requests: 0 }));
} finally { try { await playwright(['close']); } finally { await bridge.close(); } }
