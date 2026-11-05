import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';
const session = `noteka-startup-${process.pid}`;
function run(command, args) { return new Promise((done, reject) => { const child = spawn(command, args, { stdio: 'inherit' }); child.on('error', reject); child.on('exit', code => code === 0 ? done() : reject(Error(`${command}: ${code}`))); }); }
const playwright = args => run('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args]);
await run('npm', ['run', 'build']);
await run('cargo', ['build', '--offline', '-p', 'noteka-cli']);
const bridge = await startHeadlessBridge();
console.log(`STARTUP_EVIDENCE=${bridge.root}`);
const config = join(bridge.root, 'playwright.config.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block' } } }));
try {
  await playwright(['open', bridge.origin, `--config=${config}`]);
  await playwright(['run-code', `--filename=${resolve('tests/headless-startup.cjs')}`]);
  const results = JSON.parse(readFileSync(join(bridge.root, 'results.json'), 'utf8'));
  const result = results.at(-1);
  if (!result?.ok || result.checks.length < 7) throw Error(`Arranque incompleto: ${JSON.stringify(result)}`);
  console.log(`Startup: ${result.checks.length} checks; ${bridge.root}/results.json`);
} finally { try { await playwright(['close']); } finally { await bridge.close(); } }
