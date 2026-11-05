// Explicitly authorized native window/focus test; synthetic workspace, no capture.
import { spawn, execFileSync } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

if (process.env.NOTEKA_NATIVE_SCREEN !== '1') throw Error('Native focus test requires explicit screen authorization.');
if (process.platform !== 'darwin') throw Error('Native focus test requires macOS.');
const root = mkdtempSync(join(process.env.NOTEKA_TEST_TMP || tmpdir(), 'noteka-focus-native-'));
execFileSync('cargo', ['build', '--locked', '--offline', '-p', 'noteka-cli', '-p', 'noteka-mcp', '-p', 'noteka-desktop', '--features', 'noteka-desktop/native-smoke,noteka-desktop/custom-protocol'], { stdio: 'inherit' });
const binary = resolve('target/debug/noteka-desktop');
await new Promise((done, reject) => {
  const child = spawn('/usr/bin/sandbox-exec', ['-p', '(version 1) (allow default) (deny network*)', binary], {
    env: { ...process.env, NOTEKA_SMOKE_ROOT: root, NOTEKA_SMOKE_CLI: resolve('target/debug/noteka'), NOTEKA_SMOKE_MCP: resolve('target/debug/noteka-mcp'), NOTEKA_SMOKE_WRITERS: resolve('scripts/native-concurrent-writers.py'), NOTEKA_SMOKE_PHASE: 'focus', NOTEKA_SMOKE_THEME: 'dark' }
  });
  let output = '';
  const timeout = setTimeout(() => { child.kill(); reject(Error(`Native focus test timed out; fixture ${root}`)); }, 60000);
  child.stdout.on('data', chunk => { output += chunk; process.stdout.write(chunk); });
  child.stderr.on('data', chunk => process.stderr.write(chunk));
  child.on('error', reject);
  child.on('exit', (code, signal) => {
    clearTimeout(timeout);
    const line = output.split('\n').find(value => value.startsWith('NOTEKA_SMOKE_RESULT '));
    const result = line ? JSON.parse(line.slice('NOTEKA_SMOKE_RESULT '.length)) : null;
    if (result) writeFileSync(join(root, 'native-focus-result.json'), JSON.stringify(result, null, 2));
    if (code !== 0 || !result?.ok) {
      reject(Error(`Native focus failed (code=${code}, signal=${signal}, result=${!!line}); fixture ${root}`));
      return;
    }
    done();
  });
});
console.log(`Native focus evidence: ${root}`);
