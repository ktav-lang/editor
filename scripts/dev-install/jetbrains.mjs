// JetBrains IDE targets: discovery, identity checks and the install
// transaction. Old installations are moved into a backup folder next to
// the IDE's plugins directory; nothing is ever deleted.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { readZip, assertEntriesUnder } from './zip.mjs';
import { exists, isRealDir, mkdirFresh, writeFileFresh, moveDir, removeEmptyDir } from './fsguard.mjs';

export const PLUGIN_DIR = 'ktav-intellij';
export const PLUGIN_ID = 'lang.ktav';
const BACKUPS = 'ktav-dev-backups';

/** Config roots that hold `<Product><version>/plugins` directories. */
export function jetbrainsRoots(env = process.env) {
  if (env.KTAV_DEV_INSTALL_JETBRAINS_ROOTS) {
    return env.KTAV_DEV_INSTALL_JETBRAINS_ROOTS.split(path.delimiter).filter(Boolean);
  }
  const home = os.homedir();
  if (process.platform === 'win32') {
    const appData = env.APPDATA || path.join(home, 'AppData', 'Roaming');
    return [path.join(appData, 'JetBrains'), path.join(appData, 'Google')];
  }
  if (process.platform === 'darwin') {
    const support = path.join(home, 'Library', 'Application Support');
    return [path.join(support, 'JetBrains'), path.join(support, 'Google')];
  }
  const config = env.XDG_CONFIG_HOME || path.join(home, '.config');
  return [path.join(config, 'JetBrains'), path.join(config, 'Google')];
}

/** Read-only: every IDE config directory under the known roots. */
export function discoverJetbrains(env = process.env) {
  const found = [];
  for (const root of jetbrainsRoots(env)) {
    let names;
    try { names = fs.readdirSync(root, { withFileTypes: true }); } catch { continue; }
    for (const d of names) {
      if (!d.isDirectory() && !d.isSymbolicLink()) continue;
      const configDir = path.join(root, d.name);
      if (!exists(path.join(configDir, 'options')) && !exists(path.join(configDir, 'plugins'))) continue;
      const pluginsDir = path.join(configDir, 'plugins');
      let installed = null;
      const target = path.join(pluginsDir, PLUGIN_DIR);
      if (exists(target)) {
        try { installed = identifyKtavPlugin(target); } catch (e) { installed = { error: e.message }; }
      }
      found.push({ name: d.name, configDir, pluginsDir, running: ideRunning(configDir), installed });
    }
  }
  return found;
}

/** A running JetBrains IDE holds `<config>/.lock`; it is removed on exit. */
export function ideRunning(configDir) {
  return exists(path.join(configDir, '.lock'));
}

/** `NAME` (e.g. `WebStorm2025.3`) or an explicit plugins directory path. */
export function resolvePluginsDir(arg, env = process.env) {
  if (arg.includes('/') || arg.includes('\\') || path.isAbsolute(arg)) return path.resolve(arg);
  const matches = discoverJetbrains(env).filter((t) => t.name.toLowerCase() === arg.toLowerCase());
  if (matches.length !== 1) {
    throw new Error(`JetBrains IDE "${arg}" not found (${matches.length} matches); run \`list\` or pass its plugins directory`);
  }
  return matches[0].pluginsDir;
}

/** Validate a plugins directory and return its real path. */
function isInside(child, parent) {
  const rel = path.relative(parent, child);
  return rel === '' || (!rel.startsWith('..') && !path.isAbsolute(rel));
}

export function checkPluginsDir(pluginsDir, repoRoot) {
  const abs = path.resolve(pluginsDir);
  if (path.basename(abs).toLowerCase() !== 'plugins') {
    throw new Error(`refusing: target must be an IDE "plugins" directory, got ${abs}`);
  }
  const configDir = path.dirname(abs);
  if (!exists(configDir) || !fs.statSync(configDir).isDirectory()) {
    throw new Error(`IDE config directory does not exist: ${configDir}`);
  }
  let real;
  if (exists(abs)) {
    if (!fs.statSync(abs).isDirectory()) throw new Error(`not a directory: ${abs}`);
    real = fs.realpathSync(abs);
  } else {
    real = path.join(fs.realpathSync(configDir), 'plugins');
  }
  const realConfig = path.dirname(real);
  if (realConfig === path.parse(realConfig).root
      || realConfig === fs.realpathSync(os.homedir())
      || isInside(realConfig, fs.realpathSync(repoRoot))) {
    throw new Error(`refusing unsafe plugins directory: ${real}`);
  }
  if (ideRunning(realConfig)) {
    throw new Error(`IDE appears to be running (${path.join(realConfig, '.lock')} exists); close it first — this script never stops processes`);
  }
  return { pluginsDir: real, configDir: realConfig };
}

function pluginXmlOf(jarBuf) {
  const entry = readZip(jarBuf).find((e) => e.name === 'META-INF/plugin.xml');
  if (!entry) return null;
  const xml = entry.data().toString('utf8');
  const id = /<id>\s*([^<]+?)\s*<\/id>/.exec(xml)?.[1];
  const version = /<version>\s*([^<]+?)\s*<\/version>/.exec(xml)?.[1];
  return { id, version };
}

/** Prove a directory is the Ktav plugin before touching it. */
export function identifyKtavPlugin(dir) {
  if (!isRealDir(dir)) throw new Error(`not a real directory (symlink/junction or file): ${dir}`);
  const lib = path.join(dir, 'lib');
  if (!isRealDir(lib)) throw new Error(`no lib/ directory in ${dir}`);
  const jars = fs.readdirSync(lib).filter((n) => /^ktav-intellij-.*\.jar$/.test(n) && !n.endsWith('-searchableOptions.jar'));
  for (const jar of jars) {
    const info = pluginXmlOf(fs.readFileSync(path.join(lib, jar)));
    if (info?.id === PLUGIN_ID) return { version: info.version ?? '?', jar };
  }
  throw new Error(`${dir} is not the ${PLUGIN_ID} plugin (no matching plugin.xml)`);
}

/** Validate a built plugin ZIP entirely in memory. */
export function inspectPluginZip(zipPath, hostPlatform) {
  const entries = readZip(fs.readFileSync(zipPath));
  assertEntriesUnder(entries, PLUGIN_DIR);
  // Decompress and CRC-check everything before any write happens.
  for (const e of entries) if (!e.isDir) e.buf = e.data();
  const jar = entries.find((e) => /^ktav-intellij\/lib\/ktav-intellij-[^/]*\.jar$/.test(e.name) && !e.name.endsWith('-searchableOptions.jar'));
  if (!jar) throw new Error(`${zipPath}: no ktav-intellij/lib/ktav-intellij-*.jar`);
  const info = pluginXmlOf(jar.buf);
  if (info?.id !== PLUGIN_ID) throw new Error(`${zipPath}: plugin id is ${info?.id ?? 'missing'}, expected ${PLUGIN_ID}`);
  const bin = hostPlatform && entries.some((e) => e.name.startsWith(`${PLUGIN_DIR}/lib/bin/${hostPlatform.dir}/ktav-lsp`));
  return { entries, version: info.version ?? '?', hasHostBinary: Boolean(bin) };
}

/** Read-only plan: everything is checked before any write happens. */
export function planJetbrainsInstall({ target, zipPath, repoRoot, hostPlatform, env }) {
  const { pluginsDir, configDir } = checkPluginsDir(resolvePluginsDir(target, env), repoRoot);
  const installedPath = path.join(pluginsDir, PLUGIN_DIR);
  let previous = null;
  if (exists(installedPath)) previous = identifyKtavPlugin(installedPath);
  const zip = zipPath ? inspectPluginZip(zipPath, hostPlatform) : null;
  return { target, pluginsDir, configDir, installedPath, previous, zipPath, zip, backupRoot: path.join(configDir, BACKUPS) };
}

export function describeJetbrainsPlan(plan) {
  const lines = [`JetBrains: ${plan.pluginsDir}`];
  lines.push(plan.previous
    ? `  move current ${PLUGIN_DIR} (${plan.previous.version}) -> ${plan.backupRoot}${path.sep}<stamp>${path.sep}previous`
    : `  no ${PLUGIN_DIR} installed yet`);
  lines.push(`  install ${plan.zip ? `${plan.zipPath} (${plan.zip.version})` : 'the freshly built plugin ZIP'}`);
  lines.push('  other plugins, settings, caches and logs are not touched');
  return lines.join('\n');
}

function extractInto(entries, destParent) {
  for (const e of entries) {
    const rel = e.name.replace(/\/$/, '').split('/');
    const out = path.join(destParent, ...rel);
    if (e.isDir) { fs.mkdirSync(out, { recursive: true }); continue; }
    fs.mkdirSync(path.dirname(out), { recursive: true });
    writeFileFresh(out, e.buf, e.mode && process.platform !== 'win32' ? e.mode : undefined);
  }
}

/** Execute a plan. Re-validates, then only moves directories. */
export function applyJetbrainsInstall(plan, { repoRoot, hostPlatform, env, stamp, log }) {
  // The IDE may have been started while we were building.
  const fresh = planJetbrainsInstall({ target: plan.pluginsDir, zipPath: plan.zipPath, repoRoot, hostPlatform, env });
  if (!fresh.zip) throw new Error('no plugin ZIP to install');
  if (!fresh.zip.hasHostBinary) log(`  warning: ZIP has no bundled ktav-lsp for ${hostPlatform?.dir}`);

  if (!exists(fresh.backupRoot)) mkdirFresh(fresh.backupRoot);
  if (!isRealDir(fresh.backupRoot)) throw new Error(`backup folder is not a real directory: ${fresh.backupRoot}`);
  const runDir = path.join(fresh.backupRoot, stamp);
  mkdirFresh(runDir);
  const staging = path.join(runDir, 'new');
  mkdirFresh(staging);
  extractInto(fresh.zip.entries, staging);
  const staged = path.join(staging, PLUGIN_DIR);
  identifyKtavPlugin(staged);

  if (!exists(fresh.pluginsDir)) mkdirFresh(fresh.pluginsDir);
  let movedPrevious = null;
  if (fresh.previous) {
    mkdirFresh(path.join(runDir, 'previous'));
    movedPrevious = path.join(runDir, 'previous', PLUGIN_DIR);
    moveDir(fresh.installedPath, movedPrevious);
  }
  try {
    moveDir(staged, fresh.installedPath);
  } catch (err) {
    if (movedPrevious && !exists(fresh.installedPath)) moveDir(movedPrevious, fresh.installedPath);
    throw err;
  }
  removeEmptyDir(staging);
  const note = [
    `Ktav dev install ${stamp}`,
    `installed: ${fresh.installedPath} (${fresh.zip.version}) from ${fresh.zipPath}`,
    movedPrevious
      ? `previous (${fresh.previous.version}) kept at: ${movedPrevious}\nto restore: close the IDE, move ${fresh.installedPath} away and move the previous folder back`
      : 'no previous installation',
    '',
  ].join('\n');
  writeFileFresh(path.join(runDir, 'INSTALL.txt'), note);
  log(`  installed ${fresh.zip.version} into ${fresh.installedPath}`);
  if (movedPrevious) log(`  previous ${fresh.previous.version} kept at ${movedPrevious}`);
  return { runDir, movedPrevious };
}
