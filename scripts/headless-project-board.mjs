import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';

const session = `noteka-project-board-${process.pid}`;
const playwright = args => run('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args]);
function run(command, args) {
  return new Promise((done, reject) => {
    const child = spawn(command, args, { stdio: 'inherit' });
    const timeout = setTimeout(() => child.kill(), 600_000);
    child.on('error', error => { clearTimeout(timeout); reject(error); });
    child.on('close', code => { clearTimeout(timeout); code === 0 ? done() : reject(new Error(`${command}: code=${code}`)); });
  });
}

await run('npm', ['run', 'build']);
await run('cargo', ['build', '--offline', '-p', 'noteka-cli']);
mkdirSync('output/playwright', { recursive: true });
const bridge = await startHeadlessBridge();
const config = join(bridge.root, 'playwright.config.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block' } } }));
try {
  await playwright(['open', bridge.origin, `--config=${config}`]);
  await playwright(['run-code', `--filename=${resolve('tests/headless-project-board.cjs')}`]);
  console.log(`Project board headless aprobado: ${bridge.root}`);
} finally {
  try { await playwright(['close']); } finally { await bridge.close(); }
}
