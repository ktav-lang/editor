import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const require = createRequire(new URL('../vscode/package.json', import.meta.url));
const parse = require('spdx-expression-parse');
const unrestricted = new Set(['MIT', 'Apache-2.0', 'BSD-2-Clause', 'BSD-3-Clause', 'ISC', '0BSD', 'Zlib']);
const developmentOnly = new Set(['BlueOak-1.0.0', 'Artistic-2.0', 'CC-BY-3.0', 'EPL-2.0', 'Python-2.0', 'WTFPL']);
const reviewedSigningLicense = '80780388a077c74609a5fc7ee1cc3d382740a5e48d87f7aac5a20cbddef3e91c';
const platformNames = new Set(['alpine-arm64', 'alpine-x64', 'darwin-arm64', 'darwin-x64', 'linux-arm', 'linux-arm64', 'linux-x64', 'win32-arm64', 'win32-x64']);

export function allowedExpression(expression, dev) {
  try {
    function visit(node) {
      if (node.conjunction === 'and') return visit(node.left) && visit(node.right);
      if (node.conjunction === 'or') return visit(node.left) || visit(node.right);
      if (node.exception || node.plus) return false;
      return unrestricted.has(node.license) || (dev && developmentOnly.has(node.license));
    }
    return visit(parse(expression));
  } catch {
    return false;
  }
}

export function isReviewedSigningPackage(path, pkg) {
  const prefix = 'node_modules/@vscode/vsce-sign';
  if (pkg?.dev !== true || pkg?.license !== 'SEE LICENSE IN LICENSE.txt') return false;
  if (path === prefix) return pkg.version === '2.0.9';
  if (!path.startsWith(`${prefix}-`)) return false;
  return pkg.version === '2.0.6' && platformNames.has(path.slice(`${prefix}-`.length));
}

export function inspectLock(lock) {
  if (lock?.lockfileVersion !== 3 || !lock.packages || typeof lock.packages !== 'object' || Array.isArray(lock.packages) || !lock.packages['']) {
    throw new Error('Invalid npm lockfile');
  }
  const failures = [];
  const restricted = [];
  for (const [path, pkg] of Object.entries(lock.packages)) {
    if (!path) continue;
    if (path === 'node_modules/@vscode/vsce-sign' || path.startsWith('node_modules/@vscode/vsce-sign-')) {
      if (isReviewedSigningPackage(path, pkg)) restricted.push({ path, ...pkg });
      else failures.push(`${path}: signing package requires review`);
    } else if (typeof pkg?.license !== 'string' || !allowedExpression(pkg.license, pkg.dev === true)) {
      failures.push(`${path}: unapproved or missing license ${JSON.stringify(pkg?.license)}`);
    }
  }
  return { failures, restricted };
}

export function verifySigningBytes(pkg, archive, licenseText) {
  if (typeof pkg.integrity !== 'string' || !pkg.integrity.startsWith('sha512-')) return false;
  const digest = createHash('sha512').update(archive).digest('base64');
  return pkg.integrity === `sha512-${digest}` && createHash('sha256').update(licenseText).digest('hex') === reviewedSigningLicense;
}

async function main() {
  const vscode = JSON.parse(await readFile('vscode/package-lock.json', 'utf8'));
  const docs = JSON.parse(await readFile('polydoc/package-lock.json', 'utf8'));
  const parser = vscode.packages?.['node_modules/spdx-expression-parse'];
  if (parser?.version !== '3.0.1' || parser?.license !== 'MIT') throw new Error('SPDX parser changed; review it first');
  const failures = [];
  const { failures: vscodeFailures, restricted } = inspectLock(vscode);
  failures.push(...vscodeFailures, ...inspectLock(docs).failures);
  for (const pkg of restricted) {
    try {
      const url = new URL(pkg.resolved);
      if (url.protocol !== 'https:' || url.hostname !== 'registry.npmjs.org') throw new Error('untrusted registry URL');
      const response = await fetch(url, { redirect: 'error', signal: AbortSignal.timeout(30000) });
      if (!response.ok) throw new Error(`registry HTTP ${response.status}`);
      const chunks = [];
      let size = 0;
      for await (const chunk of response.body) {
        size += chunk.length;
        if (size > 64 * 1024 * 1024) throw new Error('archive exceeds size limit');
        chunks.push(chunk);
      }
      const archive = Buffer.concat(chunks);
      const result = spawnSync('tar', ['-xOzf', '-', 'package/LICENSE.txt'], { input: archive, maxBuffer: 1024 * 1024 });
      if (result.status !== 0 || !verifySigningBytes(pkg, archive, result.stdout)) throw new Error('archive integrity or reviewed license changed');
      const metadata = spawnSync('tar', ['-xOzf', '-', 'package/package.json'], { input: archive, maxBuffer: 1024 * 1024 });
      if (metadata.status !== 0) throw new Error('package metadata missing');
      const actual = JSON.parse(metadata.stdout.toString('utf8'));
      if (actual.name !== pkg.path.slice('node_modules/'.length) || actual.version !== pkg.version || actual.license !== pkg.license) {
        throw new Error('package metadata differs from lockfile');
      }
    } catch (error) {
      failures.push(`${pkg.path}: ${error.message}`);
    }
  }
  if (failures.length) {
    for (const failure of failures) console.error(failure);
    process.exitCode = 1;
  } else {
    console.log(`Checked ${Object.keys(vscode.packages).length - 1} VS Code and ${Object.keys(docs.packages).length - 1} docs lock entries`);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await main();
