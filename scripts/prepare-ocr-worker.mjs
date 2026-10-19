// Compila únicamente código local con el SDK existente; no descarga recursos.
import { execFileSync } from 'node:child_process';
import { mkdirSync, realpathSync, lstatSync, copyFileSync, mkdtempSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import os from 'node:os';

const root = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const args = process.argv.slice(2);
if (args.some(a => !['--release', '--probes', '--tauri'].includes(a))) throw new Error('Opciones: --release, --probes, --tauri');
if (process.platform !== 'darwin') throw new Error('OCR requiere macOS y un SDK ya instalado.');
const tauri = args.includes('--tauri');
const release = args.includes('--release') || (tauri && process.env.TAURI_ENV_DEBUG === 'false'), probes = args.includes('--probes');
if (tauri && probes) throw new Error('Los probes no se distribuyen.');
const host = execFileSync('rustc', ['-vV'], { encoding: 'utf8', cwd: root }).match(/^host: (.+)$/m)[1];
const triple = (tauri && process.env.TAURI_ENV_TARGET_TRIPLE) || process.env.CARGO_BUILD_TARGET || host;
if (!/^(aarch64|x86_64)-apple-darwin$/.test(triple)) throw new Error('Target OCR no admitido');
const sdk = realpathSync(execFileSync('/usr/bin/xcrun', ['--sdk', 'macosx', '--show-sdk-path'], { encoding: 'utf8' }).trim());
// swiftc selecciona el modo del driver por argv[0]; validar el destino del enlace
// sin invocar swift-frontend directamente.
const swift = execFileSync('/usr/bin/xcrun', ['--find', 'swiftc'], { encoding: 'utf8' }).trim();
if (!lstatSync(sdk).isDirectory() || !lstatSync(realpathSync(swift)).isFile()) throw new Error('SDK/Swift no disponibles.');
const temporary = process.env.NOTEKA_TEST_TMP || mkdtempSync(path.join(os.tmpdir(), 'noteka-ocr-build-'));
if (!temporary || !lstatSync(temporary).isDirectory()) throw new Error('NOTEKA_TEST_TMP externo existente es obligatorio.');
const external = realpathSync(temporary);
if (external === root || external.startsWith(root + path.sep)) throw new Error('Caché de módulos debe estar fuera del repositorio.');
const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--offline', '--no-deps', '--format-version', '1'], { cwd: root, encoding: 'utf8' }));
const directory = path.join(metadata.target_directory, ...(triple !== host || process.env.CARGO_BUILD_TARGET ? [triple] : []), release ? 'release' : 'debug');
mkdirSync(directory, { recursive: true });
const cache = path.join(external, 'swift-module-cache'); mkdirSync(cache, { recursive: true });
const target = `${triple.startsWith('aarch64-') ? 'arm64' : 'x86_64'}-apple-macosx13.0`;
const output = path.join(directory, probes ? 'noteka-ocr-worker-probes' : 'noteka-ocr-worker');
// Plugins del SDK no pueden crear un segundo sandbox dentro del deny-network
// usado en verificación. Sólo desactiva ese sandbox del compilador; la política
// externa y la contención del helper ejecutable se conservan.
execFileSync(swift, ['-sdk', sdk, '-target', target, '-swift-version', '6', '-disable-sandbox', '-module-cache-path', cache,
  ...(release ? ['-O'] : ['-Onone']), ...(probes ? ['-D', 'NOTEKA_OCR_TEST'] : []),
  path.join(root, 'native/ocr-worker/main.swift'), '-o', output], { cwd: root, stdio: 'inherit' });
if (tauri) {
  const staging = path.join(root, 'src-tauri/binaries'); mkdirSync(staging, { recursive: true });
  const destination = path.join(staging, `noteka-ocr-worker-${triple}`);
  try { if (!lstatSync(destination).isFile()) throw new Error('Staging OCR no regular'); } catch (e) { if (e.code !== 'ENOENT') throw e; }
  copyFileSync(output, destination);
}
console.log(JSON.stringify({ output, triple, tauri, profile: release ? 'release' : 'debug', probes, target, sdk, cache }));
