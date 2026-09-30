#!/usr/bin/env node
// Build the Ktav editor plugins from this checkout and install them into
// explicitly selected IDEs.
//
// Safety contract (enforced by scripts/tests/dev-install.test.mjs):
// - Dry run by default: without --apply nothing is built or written.
// - Nothing is ever deleted. An existing JetBrains `ktav-intellij` folder is
//   first proven to be the lang.ktav plugin, then *moved* into
//   `<ide-config>/ktav-dev-backups/<stamp>/previous/`; backups are kept.
// - Only `<plugins>/ktav-intellij` is replaced; other plugins, settings,
//   caches and logs are never touched. Symlinks/junctions are refused.
// - A running JetBrains IDE (its `<config>/.lock` exists) is refused; no
//   process is ever stopped or started.
// - VS Code-family editors are updated only through their own
//   `--install-extension` CLI.
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { exists } from './dev-install/fsguard.mjs';
import { buildIntellij, buildLsp, buildVsix, hostPlatform, outputDir } from './dev-install/build.mjs';
import {
  applyJetbrainsInstall, describeJetbrainsPlan, discoverJetbrains, planJetbrainsInstall,
} from './dev-install/jetbrains.mjs';
import { applyVscodeInstall, describeVscodePlan, discoverVscode, planVscodeInstall } from './dev-install/vscode.mjs';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const USAGE = `Usage:
  node scripts/dev-install.mjs list      read-only: IDEs, CLIs, installed Ktav
  node scripts/dev-install.mjs build     build ktav-lsp, VSIX and plugin ZIP into
                                         tmp/dev-install/<stamp>/ (no install)
  node scripts/dev-install.mjs install [targets] [artifacts] [--apply]

Targets (repeatable, at least one):
  --jetbrains NAME|DIR   IDE config name from \`list\` (e.g. WebStorm2025.3)
                         or that IDE's "plugins" directory
  --vscode CLI           code, codium, code-insiders, cursor, or a CLI path

Artifacts (optional; skip the matching build):
  --plugin-zip PATH      IntelliJ plugin ZIP to install
  --vsix PATH            VSIX to install

  --apply                Actually build and install. Without it the script
                         only validates the targets and prints the plan.

Close each JetBrains IDE before --apply. Nothing is deleted: a previous
Ktav plugin is moved to <ide-config>/ktav-dev-backups/<stamp>/previous/.`;

function parseArgs(argv) {
  const opts = { command: argv[0], jetbrains: [], vscode: [], apply: false, pluginZip: null, vsix: null };
  for (let i = 1; i < argv.length; i++) {
    const a = argv[i];
    const value = () => {
      if (i + 1 >= argv.length || argv[i + 1].startsWith('--')) throw new Error(`missing value for ${a}`);
      return argv[++i];
    };
    if (a === '--apply') opts.apply = true;
    else if (a === '--jetbrains') opts.jetbrains.push(value());
    else if (a === '--vscode') opts.vscode.push(value());
    else if (a === '--plugin-zip') opts.pluginZip = path.resolve(value());
    else if (a === '--vsix') opts.vsix = path.resolve(value());
    else throw new Error(`unknown argument: ${a}`);
  }
  return opts;
}

function stampNow() {
  const d = new Date();
  const p = (n) => String(n).padStart(2, '0');
  return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}-${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}-${process.pid}`;
}

function list(log) {
  log('JetBrains IDEs (config directories):');
  const jb = discoverJetbrains();
  if (jb.length === 0) log('  none found');
  for (const t of jb) {
    const ktav = t.installed ? (t.installed.error ? `unrecognised ktav-intellij: ${t.installed.error}` : `Ktav ${t.installed.version}`) : 'Ktav not installed';
    log(`  ${t.name.padEnd(24)} ${ktav}${t.running ? '  [running]' : ''}`);
    log(`  ${''.padEnd(24)} ${t.pluginsDir}`);
  }
  log('VS Code-family CLIs on PATH:');
  const vs = discoverVscode();
  if (vs.length === 0) log('  none found');
  for (const t of vs) log(`  ${t.name.padEnd(24)} ktav ${t.version ?? 'not installed'}  (${t.cli})`);
}

function install(opts, log) {
  if (opts.jetbrains.length === 0 && opts.vscode.length === 0) throw new Error('select at least one --jetbrains or --vscode target');
  for (const p of [opts.pluginZip, opts.vsix]) if (p && !exists(p)) throw new Error(`artifact not found: ${p}`);
  const host = hostPlatform();
  const ctx = { repoRoot: REPO_ROOT, hostPlatform: host, env: process.env };

  // 1. Validate every target read-only before building anything.
  const jbPlans = opts.jetbrains.map((target) => planJetbrainsInstall({ ...ctx, target, zipPath: opts.pluginZip }));
  if (new Set(jbPlans.map((p) => p.pluginsDir.toLowerCase())).size !== jbPlans.length) {
    throw new Error('the same JetBrains plugins directory was selected twice');
  }
  const vsPlans = opts.vscode.map((target) => planVscodeInstall(target));
  for (const p of jbPlans) log(describeJetbrainsPlan(p));
  for (const p of vsPlans) log(describeVscodePlan(p, opts.vsix));
  const needZip = jbPlans.length > 0 && !opts.pluginZip;
  const needVsix = vsPlans.length > 0 && !opts.vsix;
  if (needZip || needVsix) log(`build: ktav-lsp for ${host.dir}${needVsix ? ', VSIX' : ''}${needZip ? ', IntelliJ ZIP' : ''} into tmp/dev-install/<stamp>/`);
  if (!opts.apply) {
    log('\nDRY RUN: nothing was built or changed. Re-run with --apply.');
    return;
  }

  // 2. Build what is missing.
  const stamp = stampNow();
  let zip = opts.pluginZip;
  let vsix = opts.vsix;
  if (needZip || needVsix) {
    const out = outputDir(REPO_ROOT, stamp);
    buildLsp(REPO_ROOT, host, log);
    if (needVsix) vsix = buildVsix(REPO_ROOT, out, log);
    if (needZip) zip = buildIntellij(REPO_ROOT, out, log);
    log(`artifacts: ${out}`);
  }

  // 3. Install; each JetBrains step re-validates (the IDE may have started).
  for (const p of jbPlans) {
    log(`install: ${p.pluginsDir}`);
    applyJetbrainsInstall({ ...p, zipPath: zip }, { ...ctx, stamp, log });
  }
  for (const p of vsPlans) {
    log(`install: ${p.cli}`);
    applyVscodeInstall(p, vsix, log);
  }
  log('\nDone. Restart the JetBrains IDEs and reload VS Code windows.');
}

function buildOnly(log) {
  const host = hostPlatform();
  const out = outputDir(REPO_ROOT, stampNow());
  buildLsp(REPO_ROOT, host, log);
  log(`artifact: ${buildVsix(REPO_ROOT, out, log)}`);
  log(`artifact: ${buildIntellij(REPO_ROOT, out, log)}`);
}

export function main(argv, log = console.log) {
  const opts = parseArgs(argv);
  if (opts.command === 'list') return list(log);
  if (opts.command === 'build') return buildOnly(log);
  if (opts.command === 'install') return install(opts, log);
  log(USAGE);
  if (opts.command !== undefined && opts.command !== '--help' && opts.command !== 'help') throw new Error(`unknown command: ${opts.command}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main(process.argv.slice(2));
  } catch (err) {
    console.error(`ERROR: ${err.message}`);
    process.exitCode = 1;
  }
}
