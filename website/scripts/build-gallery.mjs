import { execFileSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const output = fileURLToPath(new URL('../public/wasm/', import.meta.url));
const rawToolchain = (process.env.RUSTUP_TOOLCHAIN || 'nightly-2026-10-31').trim();
const toolchain = rawToolchain.startsWith('+') ? rawToolchain : `+${rawToolchain}`;
const target = 'wasm32-unknown-unknown';
const release = process.argv.includes('--release');
const wasmBindgen = process.env.WASM_BINDGEN || 'wasm-bindgen';

execFileSync(wasmBindgen, ['--version'], { cwd: root, stdio: 'inherit' });

const cargo = (args) => execFileSync('cargo', [toolchain, ...args], { cwd: root, encoding: 'utf8' });
const { target_directory: targetDirectory } = JSON.parse(cargo(['metadata', '--locked', '--no-deps', '--format-version=1']));

execFileSync('cargo', [
  toolchain, 'rustc', '--locked', '-p', 'gpui-material-gallery-web', '--target', target,
  ...(release ? ['--profile', 'wasm-release'] : []), '--', '-C', 'link-arg=-zstack-size=8388608',
], { cwd: root, stdio: 'inherit' });

mkdirSync(output, { recursive: true });
execFileSync(wasmBindgen, [
  join(targetDirectory, target, release ? 'wasm-release' : 'debug', 'gpui_material_gallery_web.wasm'),
  '--target', 'web', '--no-typescript', '--out-name', 'material-gallery', '--out-dir', output,
], { cwd: root, stdio: 'inherit' });
