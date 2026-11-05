// Browser + test bridge + real CLI over a synthetic workspace; no personal data.
import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';

const reports = [];
for (const size of [1000, 8000]) {
  const session = `noteka-refresh-benchmark-${process.pid}-${size}`;
  const run = (command, args) => new Promise((done, reject) => {
    const child = spawn(command, args, { stdio: 'inherit' });
    child.on('error', reject);
    child.on('exit', code => code === 0 ? done() : reject(Error(`${command}: ${code}`)));
  });
  const playwright = args => run('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args]);
  const bridge = await startHeadlessBridge();
  const config = join(bridge.root, 'playwright.config.json');
  writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block' } } }));
  try {
    await playwright(['open', `${bridge.origin}/#refreshBenchmark=${size}`, `--config=${config}`]);
    await playwright(['run-code', `--filename=${resolve('tests/headless-refresh-benchmark.cjs')}`]);
    const result = JSON.parse(readFileSync(join(bridge.root, 'results.json'), 'utf8'));
    if (result.length !== 1 || !result[0].ok || result[0].benchmark.samples.length !== 30) throw Error('Incomplete refresh benchmark');
    reports.push({ ...result[0].benchmark, evidence: bridge.root });
  } finally {
    try { await playwright(['close']); } finally { await bridge.close(); }
  }
}
const output = JSON.stringify({ reports, scope: 'headless browser + local CLI test bridge, synthetic focus events; not native Tauri IPC' }, null, 2);
if (process.env.NOTEKA_BENCH_OUTPUT) writeFileSync(process.env.NOTEKA_BENCH_OUTPUT, output + '\n');
console.log(output);
