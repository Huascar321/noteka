// Compila únicamente código local con el SDK existente; no descarga recursos.
import { execFileSync } from 'node:child_process';
import { mkdirSync, realpathSync, lstatSync, mkdtempSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import os from 'node:os';

const root = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const args = process.argv.slice(2);
if (args.some(a => !['--release', '--probes'].includes(a))) throw new Error('Opciones: --release, --probes');
if (process.platform !== 'darwin') throw new Error('PDFKit requiere macOS y un SDK ya instalado.');
const release = args.includes('--release'), probes = args.includes('--probes');
const host = execFileSync('rustc', ['-vV'], { encoding: 'utf8', cwd: root }).match(/^host: (.+)$/m)[1];
const triple = process.env.CARGO_BUILD_TARGET || host;
if (!/^(aarch64|x86_64)-apple-darwin$/.test(triple)) throw new Error('Target PDFKit no admitido');
const sdk = realpathSync(execFileSync('/usr/bin/xcrun', ['--sdk', 'macosx', '--show-sdk-path'], { encoding: 'utf8' }).trim());
// swiftc selecciona el modo del driver por argv[0]; validar el destino del enlace
// sin invocar swift-frontend directamente.
const swift = execFileSync('/usr/bin/xcrun', ['--find', 'swiftc'], { encoding: 'utf8' }).trim();
if (!lstatSync(sdk).isDirectory() || !lstatSync(realpathSync(swift)).isFile()) throw new Error('SDK/Swift no disponibles.');
const temporary = process.env.NOTEKA_TEST_TMP || mkdtempSync(path.join(os.tmpdir(), 'noteka-pdfkit-build-'));
if (!temporary || !lstatSync(temporary).isDirectory()) throw new Error('NOTEKA_TEST_TMP externo existente es obligatorio.');
const external = realpathSync(temporary);
if (external === root || external.startsWith(root + path.sep)) throw new Error('Caché de módulos debe estar fuera del repositorio.');
const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--offline', '--no-deps', '--format-version', '1'], { cwd: root, encoding: 'utf8' }));
const directory = path.join(metadata.target_directory, ...(triple !== host || process.env.CARGO_BUILD_TARGET ? [triple] : []), release ? 'release' : 'debug');
mkdirSync(directory, { recursive: true });
const cache = path.join(external, 'swift-module-cache'); mkdirSync(cache, { recursive: true });
const target = `${triple.startsWith('aarch64-') ? 'arm64' : 'x86_64'}-apple-macosx13.0`;
const output = path.join(directory, probes ? 'noteka-pdfkit-worker-probes' : 'noteka-pdfkit-worker');
try {
  if (!lstatSync(output).isFile() || lstatSync(output).isSymbolicLink()) throw new Error('Salida PDFKit debe ser un archivo regular.');
} catch (error) { if (error.code !== 'ENOENT') throw error; }
// Plugins del SDK no pueden crear un segundo sandbox dentro del deny-network
// usado en verificación. Sólo desactiva ese sandbox del compilador; la política
// externa y la contención del helper ejecutable se conservan.
execFileSync(swift, ['-sdk', sdk, '-target', target, '-swift-version', '6', '-disable-sandbox', '-module-cache-path', cache,
  ...(release ? ['-O'] : ['-Onone']), ...(probes ? ['-D', 'NOTEKA_PDFKIT_TEST'] : []),
  path.join(root, 'native/pdfkit-worker/main.swift'), '-o', output], { cwd: root, stdio: 'inherit' });
console.log(JSON.stringify({ output, triple, profile: release ? 'release' : 'debug', probes, target, sdk, cache }));
