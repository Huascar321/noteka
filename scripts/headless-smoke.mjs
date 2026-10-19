import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';

// CLI de Playwright, sesión efímera propia y headless explícito. Nunca perfil personal.
const session = `noteka-headless-${process.pid}`;
const playwright = args => run('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args]);
function run(command, args) {
  return new Promise((done, reject) => {
    const child = spawn(command, args, { stdio: 'inherit' });
    const timeout = setTimeout(() => child.kill(), 1_200_000);
    child.on('error', error => { clearTimeout(timeout); reject(error); });
    child.on('close', (code, signal) => {
      clearTimeout(timeout);
      if (code === 0) done(); else reject(new Error(`${command}: code=${code}, signal=${signal}`));
    });
  });
}

await run('npm', ['run', 'build']);
await run('cargo', ['build', '--offline', '-p', 'noteka-cli']);
const bridge = await startHeadlessBridge();
const config = join(bridge.root, 'playwright.config.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block' } } }));
try {
  await playwright(['open', bridge.origin, `--config=${config}`]);
  await playwright(['run-code', `--filename=${resolve('tests/headless-run.cjs')}`]);
  const results = JSON.parse(readFileSync(join(bridge.root, 'results.json'), 'utf8'));
  if (results.length !== 6 || results.some(result => !result.ok)) throw new Error('Recorrido headless incompleto/fallido');
  const keyboard = ['dark','light'].map(theme => JSON.parse(readFileSync(join(bridge.root, theme, 'keyboard-results.json'), 'utf8')));
  const network = JSON.parse(readFileSync(join(bridge.root, 'network-results.json'), 'utf8'));
  if (keyboard.some(r=>!r.ok) || network.external_origins.length) throw new Error('Teclado/red fallaron');
  console.log(`Headless CLI/SQLite: ${results.reduce((n, r) => n + r.checks.length, 0)} checks + ${keyboard.reduce((n,r)=>n+r.checks.length,0)} de teclado; ${bridge.root}/results.json`);
} finally {
  try { await playwright(['close']); } finally { await bridge.close(); }
}
