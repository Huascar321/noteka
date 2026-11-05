import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';
const session = `noteka-goal-detail-${process.pid}`;
function run(command, args) { return new Promise((done, reject) => { const child = spawn(command, args, { stdio: 'inherit' }); child.on('error', reject); child.on('exit', code => code === 0 ? done() : reject(Error(`${command}: ${code}`))); }); }
const playwright = args => run('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args]);
const bridge = await startHeadlessBridge(); const config = join(bridge.root, 'playwright.config.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block' } } }));
try {
  await playwright(['open', `${bridge.origin}/?testTheme=${process.env.NOTEKA_TEST_THEME === 'dark' ? 'dark' : 'light'}`, `--config=${config}`, ...(process.env.NOTEKA_TEST_BROWSER ? [`--browser=${process.env.NOTEKA_TEST_BROWSER}`] : [])]);
  await playwright(['run-code', `--filename=${resolve('tests/headless-goal-detail.cjs')}`]);
  const result = JSON.parse(readFileSync(join(bridge.root, 'results.json'), 'utf8'));
  if (result.length !== 1 || !result[0].ok) throw Error('Incomplete Goal detail checks');
  console.log(`Goal detail: ${result[0].checks.length} checks passed. Evidence: ${bridge.root}`);
} finally { try { await playwright(['close']); } finally { await bridge.close(); } }
