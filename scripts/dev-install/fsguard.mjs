// The only filesystem mutations dev-install may perform. There is no
// recursive delete here on purpose: existing data is moved, never removed.
// A test asserts that no other module touches the filesystem destructively.
import fs from 'node:fs';
import path from 'node:path';

export function exists(p) {
  try { fs.lstatSync(p); return true; } catch (e) { if (e.code === 'ENOENT') return false; throw e; }
}

/** Real directory, not a symlink or junction. */
export function isRealDir(p) {
  try { const s = fs.lstatSync(p); return s.isDirectory() && !s.isSymbolicLink(); } catch { return false; }
}

/** Create one new directory; fails if it already exists. */
export function mkdirFresh(p) {
  fs.mkdirSync(p);
}

/** Create a new file; fails if anything already exists at `p`. */
export function writeFileFresh(p, data, mode) {
  fs.writeFileSync(p, data, { flag: 'wx', mode });
}

/**
 * Atomic same-volume move. Refuses when the destination exists (POSIX
 * rename would otherwise replace an empty directory silently).
 */
export function moveDir(from, to) {
  if (exists(to)) throw new Error(`refusing to move onto existing path: ${to}`);
  if (!isRealDir(from)) throw new Error(`not a real directory: ${from}`);
  fs.renameSync(from, to);
}

/** Remove a directory only if it is empty (never recursive). */
export function removeEmptyDir(p) {
  fs.rmdirSync(p);
}

/** Copy a build artifact over a previous build artifact inside the repo. */
export function copyBuildFile(from, to) {
  const dir = path.dirname(to);
  fs.mkdirSync(dir, { recursive: true });
  if (fs.lstatSync(dir).isSymbolicLink()) throw new Error(`refusing symlinked build directory: ${dir}`);
  if (exists(to) && !fs.lstatSync(to).isFile()) throw new Error(`refusing to overwrite non-file: ${to}`);
  fs.copyFileSync(from, to);
}
