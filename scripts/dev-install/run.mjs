// Child-process helper: argument arrays only, no shell string building
// except for Windows .cmd/.bat launchers, whose arguments are validated.
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const CMD_UNSAFE = /["%!^&|<>\r\n]/;

export function run(cmd, args, { cwd, capture = false, env } = {}) {
  const opts = { cwd, env: env ?? process.env, encoding: 'utf8', stdio: capture ? ['ignore', 'pipe', 'pipe'] : 'inherit' };
  let res;
  if (process.platform === 'win32' && /\.(cmd|bat)$/i.test(cmd)) {
    for (const a of [cmd, ...args]) {
      if (CMD_UNSAFE.test(a)) throw new Error(`refusing to pass unsafe argument to ${path.basename(cmd)}: ${a}`);
    }
    const line = [cmd, ...args].map((a) => `"${a}"`).join(' ');
    res = spawnSync(process.env.ComSpec || 'cmd.exe', ['/d', '/s', '/c', `"${line}"`], { ...opts, windowsVerbatimArguments: true });
  } else {
    res = spawnSync(cmd, args, opts);
  }
  if (res.error) throw new Error(`${path.basename(cmd)}: ${res.error.message}`);
  if (res.status !== 0) {
    const detail = capture ? `\n${res.stderr || res.stdout}` : '';
    throw new Error(`${path.basename(cmd)} ${args.join(' ')} exited with ${res.status}${detail}`);
  }
  return res.stdout ?? '';
}

/** Resolve an executable name on PATH (with PATHEXT on Windows) or a path. */
export function which(name, env = process.env) {
  if (name.includes('/') || name.includes('\\')) return fs.existsSync(name) ? path.resolve(name) : null;
  const exts = process.platform === 'win32' ? ['.cmd', '.exe', '.bat'] : [''];
  for (const dir of (env.PATH || env.Path || '').split(path.delimiter)) {
    if (!dir) continue;
    for (const ext of exts) {
      const p = path.join(dir, name + ext);
      try { if (fs.statSync(p).isFile()) return p; } catch { /* next */ }
    }
  }
  return null;
}
