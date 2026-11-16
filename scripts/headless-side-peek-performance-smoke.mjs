import { spawn } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';
const browser = process.env.NOTEKA_TEST_BROWSER || 'chromium';
const stage = process.env.NOTEKA_PERF_STAGE || 'after';
const output = resolve('output/playwright/side-peek-performance');
mkdirSync(output, { recursive: true });
const bridge = await startHeadlessBridge(), session = `peek-perf-${process.pid}`;
const config = join(bridge.root, 'playwright.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block', deviceScaleFactor: 2, reducedMotion: 'no-preference' } } }));
const run = args => new Promise((done, reject) => {
  const child = spawn('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args], { stdio: 'inherit' });
  child.on('error', reject); child.on('exit', code => code === 0 ? done() : reject(Error(`Playwright: ${code}`)));
});
try {
  await run(['open', `${bridge.origin}#${stage}`, `--config=${config}`, ...(browser === 'webkit' ? ['--browser=webkit'] : [])]);
  await run(['run-code', `--filename=${resolve('tests/headless-side-peek-performance.cjs')}`]);
  const result = JSON.parse(readFileSync(join(bridge.root, 'results.json'))).at(-1);
  writeFileSync(join(output, `${stage}-${browser}.json`), JSON.stringify(result, null, 2) + '\n');
  if (!result?.ok) throw Error('Incomplete side peek checks');
  console.log(`Side peek: ${result.checks.length} checks passed; ${JSON.stringify(result.samples)}`);
} finally { try { await run(['close']); } finally { await bridge.close(); } }
