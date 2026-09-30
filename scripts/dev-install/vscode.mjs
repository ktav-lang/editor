// VS Code-family targets. Installation goes through the editor's own
// `--install-extension` CLI; this script never edits extension folders.
import { run, which } from './run.mjs';

export const EXTENSION_ID = 'ktav-lang.ktav';
export const KNOWN_CLIS = ['code', 'code-insiders', 'codium', 'cursor'];

export function installedVersion(cli) {
  const out = run(cli, ['--list-extensions', '--show-versions'], { capture: true });
  const line = out.split(/\r?\n/).find((l) => l.toLowerCase().startsWith(`${EXTENSION_ID}@`));
  return line ? line.slice(EXTENSION_ID.length + 1).trim() : null;
}

/** Read-only: CLIs on PATH and the Ktav version each one has. */
export function discoverVscode(env = process.env) {
  const found = [];
  for (const name of KNOWN_CLIS) {
    const cli = which(name, env);
    if (!cli) continue;
    let version = null;
    try { version = installedVersion(cli); } catch (e) { version = `error: ${e.message.split('\n')[0]}`; }
    found.push({ name, cli, version });
  }
  return found;
}

export function planVscodeInstall(target, env = process.env) {
  const cli = which(target, env);
  if (!cli) throw new Error(`VS Code CLI "${target}" not found on PATH; pass its full path`);
  return { target, cli, previous: installedVersion(cli) };
}

export function describeVscodePlan(plan, vsix) {
  return [
    `VS Code: ${plan.cli}`,
    `  current ${EXTENSION_ID}: ${plan.previous ?? 'not installed'}`,
    `  run: ${plan.target} --install-extension ${vsix ?? '<freshly built VSIX>'} --force`,
    '  the editor replaces its own copy; reload open windows afterwards',
  ].join('\n');
}

export function applyVscodeInstall(plan, vsix, log) {
  run(plan.cli, ['--install-extension', vsix, '--force'], { capture: true });
  log(`  ${plan.target}: ${EXTENSION_ID} now ${installedVersion(plan.cli) ?? 'NOT FOUND'}`);
}
