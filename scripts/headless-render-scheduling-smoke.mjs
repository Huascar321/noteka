import { spawn } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';

const browser = process.env.NOTEKA_TEST_BROWSER || 'chromium';
const output = resolve('output/playwright/render-scheduling', browser);
mkdirSync(output, { recursive: true });
// The test captures only a synthetic workspace, never native user content.
mkdirSync(resolve('output/playwright/performance'), { recursive: true });
const bridge = await startHeadlessBridge();
const session = `noteka-render-${process.pid}`, config = join(bridge.root, 'playwright.json');
writeFileSync(config, JSON.stringify({ browser: { launchOptions: { headless: true }, contextOptions: { serviceWorkers: 'block' } } }));
const run = args => new Promise((done, reject) => {
  const child = spawn('npx', ['--yes', '--package', '@playwright/cli@0.1.22', 'playwright-cli', `-s=${session}`, ...args], { stdio: 'inherit' });
  child.on('error', reject); child.on('exit', code => code === 0 ? done() : reject(Error(`Playwright: ${code}`)));
});
try {
  await run(['open', bridge.origin, `--config=${config}`, ...(browser === 'webkit' ? ['--browser=webkit'] : [])]);
  await run(['run-code', `--filename=${resolve('tests/headless-render-scheduling.cjs')}`]);
  const result = JSON.parse(readFileSync(join(bridge.root, 'results.json'), 'utf8')).at(-1);
  if (!result?.ok) throw Error('Incomplete render checks');
  writeFileSync(join(output, 'results.json'), JSON.stringify(result, null, 2) + '\n');
  console.log(`Render scheduling: ${result.checks.length} checks passed`);
} finally { try { await run(['close']); } finally { await bridge.close(); } }
