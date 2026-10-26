import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { startHeadlessBridge } from './headless-bridge.mjs';
const session=`noteka-navigation-${process.pid}`;
function run(command,args){return new Promise((resolve,reject)=>{const child=spawn(command,args,{stdio:'inherit'});child.on('error',reject);child.on('exit',code=>code===0?resolve():reject(Error(`${command}: ${code}`)));});}
const playwright=args=>run('npx',['--yes','--package','@playwright/cli@0.1.22','playwright-cli',`-s=${session}`,...args]);
await run('npm',['run','build']);
await run('cargo',['build','--offline','-p','noteka-cli']);
const bridge=await startHeadlessBridge();
console.log(`NAVIGATION_EVIDENCE=${bridge.root}`);
const config=join(bridge.root,'playwright.config.json');
writeFileSync(config,JSON.stringify({browser:{launchOptions:{headless:true},contextOptions:{serviceWorkers:'block'}}}));
try{
 await playwright(['open',bridge.origin,`--config=${config}`]);
 await playwright(['run-code',`--filename=${resolve('tests/headless-navigation.cjs')}`]);
 const results=JSON.parse(readFileSync(join(bridge.root,'results.json'),'utf8'));
 if(results.length!==2||results.some(r=>!r.ok))throw Error('Navigation incomplete');
 console.log(`Navigation: ${results.reduce((n,r)=>n+r.checks.length,0)} checks; ${bridge.root}/results.json`);
}finally{try{await playwright(['close']);}finally{await bridge.close();}}
