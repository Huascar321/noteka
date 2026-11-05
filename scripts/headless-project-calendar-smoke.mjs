import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';
const session = `noteka-project-calendar-${process.pid}`;
const run = args => new Promise((done,reject) => { const child=spawn('npx',['--yes','--package','@playwright/cli@0.1.22','playwright-cli',`-s=${session}`,...args],{stdio:'inherit'}); child.on('error',reject); child.on('exit',code=>code===0?done():reject(Error(`Playwright: ${code}`))); });
const bridge=await startHeadlessBridge(); const config=join(bridge.root,'playwright.json');
writeFileSync(config,JSON.stringify({browser:{launchOptions:{headless:true},contextOptions:{serviceWorkers:'block',reducedMotion:'reduce'}}}));
try { await run(['open',bridge.origin,`--config=${config}`,...(process.env.NOTEKA_TEST_BROWSER?[`--browser=${process.env.NOTEKA_TEST_BROWSER}`]:[])]); await run(['run-code',`--filename=${resolve('tests/headless-project-calendar.cjs')}`]); const result=JSON.parse(readFileSync(join(bridge.root,'results.json'),'utf8')).at(-1); if(!result?.ok) throw Error('Incomplete checks'); console.log(`Project calendar: ${result.checks.length} checks passed`); }
finally { try { await run(['close']); } finally { await bridge.close(); } }
