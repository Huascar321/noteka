// Preparación local, no instalación global ni publicación. Sin shell ni red.
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync, lstatSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = fileURLToPath(new URL('..', import.meta.url));
const args = process.argv.slice(2);
const tauri = args.includes('--tauri');
if (args.some((a) => !['--tauri', '--release'].includes(a))) throw new Error('Opciones: --tauri, --release');
const release = args.includes('--release') || (tauri && process.env.TAURI_ENV_DEBUG === 'false');
const host = execFileSync('rustc', ['-vV'], { encoding: 'utf8', cwd: root }).match(/^host: (.+)$/m)[1];
const triple = (tauri && process.env.TAURI_ENV_TARGET_TRIPLE) || process.env.CARGO_BUILD_TARGET || host;
if (!/^[a-zA-Z0-9_-]+$/.test(triple)) throw new Error('Target triple inválido');
const explicit = triple !== host;
const build = ['build', '--locked', '--offline', '-p', 'noteka-pdf-worker', '-p', 'noteka-xlsx-worker', ...(release ? ['--release'] : []), ...(explicit ? ['--target', triple] : [])];
execFileSync('cargo', build, { cwd: root, stdio: 'inherit' });
const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--offline', '--no-deps', '--format-version', '1'], { cwd: root, encoding: 'utf8' }));
for (const worker of ['noteka-pdf-worker', 'noteka-xlsx-worker']) {
const executable = `${worker}${triple.includes('windows') ? '.exe' : ''}`;
const directory = path.join(metadata.target_directory, ...(explicit || process.env.CARGO_BUILD_TARGET ? [triple] : []), release ? 'release' : 'debug');
const source = path.join(directory, executable);
if (!lstatSync(source).isFile()) throw new Error('Helper no regular');
const staging = path.join(root, 'src-tauri', 'binaries');
mkdirSync(staging, { recursive: true });
const target = path.join(staging, `${worker}-${triple}${triple.includes('windows') ? '.exe' : ''}`);
try { if (!lstatSync(target).isFile()) throw new Error('Staging no regular'); } catch (error) { if (error.code !== 'ENOENT') throw error; }
copyFileSync(source, target);
console.log(JSON.stringify({ worker, profile: release ? 'release' : 'debug', triple, sibling: source, staging: target }));

}

// Vision sólo se prepara para macOS; el overlay de esa plataforma lo distribuye.
if (triple.endsWith('-apple-darwin')) {
  execFileSync(process.execPath, [path.join(root, 'scripts/prepare-ocr-worker.mjs'), ...(tauri ? ['--tauri'] : []), ...(release ? ['--release'] : [])], { cwd: root, stdio: 'inherit' });
}
