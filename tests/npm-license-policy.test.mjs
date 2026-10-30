import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { allowedExpression, dependencyFreeManifest, inspectLock, isReviewedSigningPackage, verifySigningBytes } from '../ci/check-npm-licenses.mjs';

const signingPath = 'node_modules/@vscode/vsce-sign';
const signing = { dev: true, version: '2.0.9', license: 'SEE LICENSE IN LICENSE.txt' };
const lock = (packages) => ({ lockfileVersion: 3, packages: { '': {}, ...packages } });

test('evaluates SPDX alternatives and combined obligations', () => {
  assert.equal(allowedExpression('MIT OR GPL-3.0-or-later', false), true);
  assert.equal(allowedExpression('MIT AND GPL-3.0-only', false), false);
  assert.equal(allowedExpression('MIT AND Zlib', false), true);
  assert.equal(allowedExpression('EPL-2.0', false), false);
  assert.equal(allowedExpression('EPL-2.0', true), true);
  assert.equal(allowedExpression('UNKNOWN', true), false);
});

test('limits signing-tool exception to the reviewed development packages', () => {
  assert.equal(isReviewedSigningPackage(signingPath, signing), true);
  assert.equal(isReviewedSigningPackage(signingPath, { ...signing, dev: false }), false);
  assert.equal(isReviewedSigningPackage(signingPath, { ...signing, version: '2.0.10' }), false);
  assert.equal(isReviewedSigningPackage(`${signingPath}-fake`, { ...signing, version: '2.0.6' }), false);
  assert.equal(inspectLock(lock({ [signingPath]: { ...signing, license: 'MIT' } })).failures.length, 1);
  assert.equal(inspectLock(lock({ [signingPath]: signing })).restricted.length, 1);
});

test('rejects unreviewed VSIX driver dependencies', () => {
  assert.equal(dependencyFreeManifest({}), true);
  assert.equal(dependencyFreeManifest({ dependencies: { x: '1' } }), false);
  assert.equal(dependencyFreeManifest({ workspaces: ['packages/*'] }), false);
});

test('does not accept unknown, substituted, or unverified data', () => {
  assert.equal(inspectLock(lock({ 'node_modules/x': {} })).failures.length, 1);
  assert.throws(() => inspectLock({ lockfileVersion: 2, packages: {} }));
  const archive = Buffer.from('archive');
  const integrity = `sha512-${createHash('sha512').update(archive).digest('base64')}`;
  assert.equal(verifySigningBytes({ integrity }, archive, Buffer.from('unapproved')), false);
  assert.equal(verifySigningBytes({ integrity: 'sha256-x' }, archive, Buffer.from('unapproved')), false);
});
