// Build the host ktav-lsp, the VSIX and the IntelliJ plugin ZIP from this
// checkout. Writes only into git-ignored build outputs of the repository
// and a fresh `tmp/dev-install/<stamp>/` folder.
import fs from 'node:fs';
import path from 'node:path';
import { run } from './run.mjs';
import { copyBuildFile, exists, mkdirFresh } from './fsguard.mjs';

const PLATFORMS = {
  'win32-x64': 'win32-x64', 'win32-arm64': 'win32-arm64',
  'linux-x64': 'linux-x64', 'linux-arm64': 'linux-arm64',
  'darwin-x64': 'darwin-x64', 'darwin-arm64': 'darwin-arm64',
};

export function hostPlatform() {
  const dir = PLATFORMS[`${process.platform}-${process.arch}`];
  if (!dir) throw new Error(`unsupported host platform ${process.platform}-${process.arch}`);
  return { dir, exe: process.platform === 'win32' ? 'ktav-lsp.exe' : 'ktav-lsp' };
}

export function outputDir(repoRoot, stamp) {
  const base = path.join(repoRoot, 'tmp', 'dev-install');
  fs.mkdirSync(base, { recursive: true });
  const dir = path.join(base, stamp);
  mkdirFresh(dir);
  return dir;
}

export function buildLsp(repoRoot, host, log) {
  log('build: ktav-lsp (cargo build --locked --release)');
  run('cargo', ['build', '--locked', '--release'], { cwd: path.join(repoRoot, 'lsp') });
  const bin = path.join(repoRoot, 'lsp', 'target', 'release', host.exe);
  if (!exists(bin)) throw new Error(`built binary missing: ${bin}`);
  for (const plugin of ['vscode', 'intellij']) {
    copyBuildFile(bin, path.join(repoRoot, plugin, 'bin', host.dir, host.exe));
  }
  return bin;
}

export function buildVsix(repoRoot, outDir, log) {
  const vscode = path.join(repoRoot, 'vscode');
  const vsce = path.join(vscode, 'node_modules', '@vscode', 'vsce', 'vsce');
  if (!exists(vsce)) throw new Error('vscode/node_modules is missing; run `npm ci` in vscode/ first');
  const version = JSON.parse(fs.readFileSync(path.join(vscode, 'package.json'), 'utf8')).version;
  const out = path.join(outDir, `ktav-${version}.vsix`);
  log('build: VSIX (vsce package)');
  run(process.execPath, [vsce, 'package', '--out', out], { cwd: vscode });
  if (!exists(out)) throw new Error(`VSIX missing: ${out}`);
  return out;
}

export function buildIntellij(repoRoot, outDir, log) {
  const intellij = path.join(repoRoot, 'intellij');
  const dist = path.join(intellij, 'build', 'distributions');
  const started = Date.now() - 1000;
  log('build: IntelliJ plugin (gradlew buildPlugin)');
  const gradlew = path.join(intellij, process.platform === 'win32' ? 'gradlew.bat' : 'gradlew');
  run(gradlew, ['buildPlugin', '--no-daemon'], { cwd: intellij });
  const fresh = fs.readdirSync(dist)
    .filter((n) => /^ktav-intellij-.*\.zip$/.test(n))
    .map((n) => path.join(dist, n))
    .filter((p) => fs.statSync(p).mtimeMs >= started);
  if (fresh.length !== 1) throw new Error(`expected one freshly built ZIP in ${dist}, found ${fresh.length}`);
  const out = path.join(outDir, path.basename(fresh[0]));
  fs.copyFileSync(fresh[0], out, fs.constants.COPYFILE_EXCL);
  return out;
}
