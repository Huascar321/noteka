import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';

const session = `noteka-system-theme-${process.pid}`;
function run(command, args) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, { stdio: 'inherit' });
    child.on('error', reject);
    child.on('exit', code => code === 0 ? resolvePromise() : reject(Error(`${command}: ${code}`)));
  });
}
const playwright = args => run('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args]);
const bridge = await startHeadlessBridge();
const config = join(bridge.root, 'playwright.config.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block' } } }));
try {
  await playwright(['open', bridge.origin, `--config=${config}`]);
  await playwright(['run-code', `--filename=${resolve('tests/headless-system-theme.cjs')}`]);
} finally {
  try { await playwright(['close']); } finally { await bridge.close(); }
}
