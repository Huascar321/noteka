import { execFileSync, spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';

if (process.env.NOTEKA_NATIVE_SCREEN !== '1') throw new Error('Prueba con ventana real: requiere pantalla/foco. Usá npm run test:headless. Sólo ejecutá NOTEKA_NATIVE_SCREEN=1 npm run test:native cuando el usuario autorice explícitamente usar su pantalla.');
if (process.platform !== 'darwin') throw new Error('Este recorrido está preparado para el host Tauri de macOS.');
// Todo es sintético; directorio temporal fuera del checkout y de workspaces personales.
const base = process.env.NOTEKA_TEST_TMP || tmpdir();
const root = mkdtempSync(join(base, 'noteka-native-'));
execFileSync('npm', ['run', 'build'], { stdio: 'inherit' });
execFileSync('cargo', ['build', '-p', 'noteka-cli', '-p', 'noteka-desktop', '--features', 'noteka-desktop/native-smoke,noteka-desktop/custom-protocol'], { stdio: 'inherit' });
const results = [];
for (const theme of ['dark', 'light']) {
const themeRoot = join(root, theme);
execFileSync('mkdir', ['-p', themeRoot]);
for (const phase of ['main', 'reopen', 'restored']) {
  await new Promise((done, reject) => {
    const binary = resolve('target/debug/noteka-desktop');
    const child = spawn('/usr/bin/sandbox-exec', ['-p', '(version 1) (allow default) (deny network*)', binary], { env: { ...process.env, NOTEKA_SMOKE_ROOT: themeRoot, NOTEKA_SMOKE_CLI: resolve('target/debug/noteka'), NOTEKA_SMOKE_PHASE: phase, NOTEKA_SMOKE_THEME: theme } });
    let output = '';
     const timeout = setTimeout(() => { child.kill(); reject(new Error('El recorrido nativo agotó 480 s.')); }, 480_000);
    child.stdout.on('data', chunk => { output += chunk; process.stdout.write(chunk); });
    child.stderr.on('data', chunk => process.stderr.write(chunk));
    child.on('error', reject);
    child.on('exit', (code, signal) => {
      clearTimeout(timeout);
      const line = output.split('\n').find(l => l.startsWith('NOTEKA_SMOKE_RESULT '));
      if (code !== 0 || !line) { reject(new Error(`Recorrido ${phase} falló (code=${code}, signal=${signal}, resultado=${!!line}); fixtures: ${root}`)); return; }
      const result = JSON.parse(line.slice('NOTEKA_SMOKE_RESULT '.length)); results.push({ theme, phase, ...result }); done();
    });
  });
}
}
writeFileSync(join(root, 'results.json'), JSON.stringify(results, null, 2));
console.log(`Evidencia sintética: ${root}/results.json`);
