// dev-install safety tests. Everything runs inside a throwaway sandbox
// under the OS temp directory; no real IDE, plugin or editor is touched.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test, before, after } from 'node:test';
import { fileURLToPath } from 'node:url';
import { main } from '../dev-install.mjs';
import { crc32 } from '../dev-install/zip.mjs';

const SCRIPTS = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
let sandbox;

before(() => { sandbox = fs.mkdtempSync(path.join(os.tmpdir(), 'ktav-dev-install-test-')); });
after(() => {
  // Test-owned scratch only: a direct child of the OS temp dir with our prefix.
  const real = fs.realpathSync(sandbox);
  assert.equal(path.dirname(real), fs.realpathSync(os.tmpdir()));
  assert.ok(path.basename(real).startsWith('ktav-dev-install-test-'));
  fs.rmSync(real, { recursive: true });
});

// ---- fixtures ------------------------------------------------------------

/** Build a stored (uncompressed) ZIP. entries: [name, content|null, unixMode?] */
function makeZip(entries) {
  const locals = [];
  const centrals = [];
  let offset = 0;
  for (const [name, content, mode] of entries) {
    const data = content == null ? Buffer.alloc(0) : Buffer.from(content);
    const nameBuf = Buffer.from(name);
    const crc = crc32(data);
    const loc = Buffer.alloc(30);
    loc.writeUInt32LE(0x04034b50, 0); loc.writeUInt16LE(20, 4);
    loc.writeUInt32LE(crc, 14); loc.writeUInt32LE(data.length, 18); loc.writeUInt32LE(data.length, 22);
    loc.writeUInt16LE(nameBuf.length, 26);
    locals.push(loc, nameBuf, data);
    const cen = Buffer.alloc(46);
    cen.writeUInt32LE(0x02014b50, 0); cen.writeUInt16LE((3 << 8) | 20, 4); cen.writeUInt16LE(20, 6);
    cen.writeUInt32LE(crc, 16); cen.writeUInt32LE(data.length, 20); cen.writeUInt32LE(data.length, 24);
    cen.writeUInt16LE(nameBuf.length, 28);
    cen.writeUInt32LE(((mode ?? (name.endsWith('/') ? 0o40755 : 0o100644)) << 16) >>> 0, 38);
    cen.writeUInt32LE(offset, 42);
    centrals.push(cen, nameBuf);
    offset += 30 + nameBuf.length + data.length;
  }
  const cd = Buffer.concat(centrals);
  const eocd = Buffer.alloc(22);
  eocd.writeUInt32LE(0x06054b50, 0);
  eocd.writeUInt16LE(entries.length, 8); eocd.writeUInt16LE(entries.length, 10);
  eocd.writeUInt32LE(cd.length, 12); eocd.writeUInt32LE(offset, 16);
  return Buffer.concat([...locals, cd, eocd]);
}

const pluginJar = (id, version) => makeZip([
  ['META-INF/plugin.xml', `<idea-plugin><id>${id}</id><name>Ktav</name><version>${version}</version></idea-plugin>`],
]);

function pluginZip({ version = '9.9.9', id = 'lang.ktav', extra = [] } = {}) {
  return makeZip([
    ['ktav-intellij/', null],
    ['ktav-intellij/lib/', null],
    [`ktav-intellij/lib/ktav-intellij-${version}.jar`, pluginJar(id, version)],
    ['ktav-intellij/lib/bin/linux-x64/ktav-lsp', 'lsp', 0o100755],
    ['ktav-intellij/lib/bin/win32-x64/ktav-lsp.exe', 'lsp'],
    ...extra,
  ]);
}

let counter = 0;
/** Fresh fake JetBrains root with one IDE config holding an old Ktav and neighbours. */
function makeIde({ ktav = 'valid' } = {}) {
  const root = path.join(sandbox, `case-${++counter}`, 'JetBrains');
  const config = path.join(root, 'FakeIDE2025.1');
  const plugins = path.join(config, 'plugins');
  fs.mkdirSync(path.join(config, 'options'), { recursive: true });
  fs.writeFileSync(path.join(config, 'options', 'ide.general.xml'), '<settings/>');
  fs.writeFileSync(path.join(config, 'disabled_plugins.txt'), 'some.other\n');
  fs.mkdirSync(path.join(plugins, 'other plugin', 'lib'), { recursive: true });
  fs.writeFileSync(path.join(plugins, 'other plugin', 'lib', 'other.jar'), 'OTHER');
  fs.writeFileSync(path.join(plugins, 'loose-theme.jar'), 'THEME');
  const ktavDir = path.join(plugins, 'ktav-intellij');
  if (ktav === 'valid' || ktav === 'foreign') {
    fs.mkdirSync(path.join(ktavDir, 'lib'), { recursive: true });
    fs.writeFileSync(path.join(ktavDir, 'lib', 'ktav-intellij-0.1.0.jar'),
      pluginJar(ktav === 'valid' ? 'lang.ktav' : 'com.example.other', '0.1.0'));
    fs.writeFileSync(path.join(ktavDir, 'lib', 'user-note.txt'), 'keep me');
  }
  const zips = path.join(sandbox, `case-${counter}`, 'zips');
  fs.mkdirSync(zips);
  return { root, config, plugins, ktavDir, zips, base: path.dirname(root) };
}

function writeZip(ide, name, buf) {
  const p = path.join(ide.zips, name);
  fs.writeFileSync(p, buf);
  return p;
}

/** Content hash of every file and directory under `dir` (lstat, no following). */
function snapshot(dir) {
  const out = {};
  (function walk(d) {
    for (const n of fs.readdirSync(d).sort()) {
      const p = path.join(d, n);
      const rel = path.relative(dir, p).split(path.sep).join('/');
      const st = fs.lstatSync(p);
      if (st.isSymbolicLink()) out[rel] = `link:${fs.readlinkSync(p)}`;
      else if (st.isDirectory()) { out[`${rel}/`] = 'dir'; walk(p); }
      else out[rel] = crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
    }
  })(dir);
  return out;
}

const run = (...args) => {
  const lines = [];
  main(args, (l) => lines.push(l));
  return lines.join('\n');
};

// ---- tests ---------------------------------------------------------------

test('dry run validates and changes nothing', () => {
  const ide = makeIde();
  const zip = writeZip(ide, 'good.zip', pluginZip());
  const before = snapshot(ide.base);
  const out = run('install', '--jetbrains', ide.plugins, '--plugin-zip', zip);
  assert.match(out, /DRY RUN/);
  assert.deepEqual(snapshot(ide.base), before);
});

test('apply replaces only ktav-intellij and keeps the old one as a backup', () => {
  const ide = makeIde();
  const zip = writeZip(ide, 'good.zip', pluginZip());
  const before = snapshot(ide.base);
  const oldKtav = snapshot(ide.ktavDir);
  run('install', '--jetbrains', ide.plugins, '--plugin-zip', zip, '--apply');
  const after = snapshot(ide.base);

  // Nothing outside ktav-intellij and the backup folder changed.
  for (const [rel, hash] of Object.entries(before)) {
    if (rel.includes('plugins/ktav-intellij')) continue;
    assert.equal(after[rel], hash, `changed: ${rel}`);
  }
  for (const rel of Object.keys(after)) {
    if (before[rel] !== undefined || rel.includes('plugins/ktav-intellij') || rel.includes('ktav-dev-backups')) continue;
    assert.fail(`unexpected new path: ${rel}`);
  }
  assert.ok(fs.existsSync(path.join(ide.ktavDir, 'lib', 'ktav-intellij-9.9.9.jar')));
  assert.ok(!fs.existsSync(path.join(ide.ktavDir, 'lib', 'user-note.txt')));

  const backups = path.join(ide.config, 'ktav-dev-backups');
  const [stamp] = fs.readdirSync(backups);
  assert.deepEqual(snapshot(path.join(backups, stamp, 'previous', 'ktav-intellij')), oldKtav, 'backup is byte-identical');
  assert.ok(!fs.existsSync(path.join(backups, stamp, 'new')), 'empty staging folder removed');
  assert.match(fs.readFileSync(path.join(backups, stamp, 'INSTALL.txt'), 'utf8'), /previous \(0\.1\.0\) kept at/);
  if (process.platform !== 'win32') {
    assert.ok(fs.statSync(path.join(ide.ktavDir, 'lib', 'bin', 'linux-x64', 'ktav-lsp')).mode & 0o100);
  }
});

test('a second apply keeps both earlier installations', () => {
  const ide = makeIde();
  run('install', '--jetbrains', ide.plugins, '--plugin-zip', writeZip(ide, 'a.zip', pluginZip({ version: '1.0.0' })), '--apply');
  // Distinct stamps come from the pid+second; force a different second.
  const wait = Date.now() + 1100; while (Date.now() < wait) { /* spin */ }
  run('install', '--jetbrains', ide.plugins, '--plugin-zip', writeZip(ide, 'b.zip', pluginZip({ version: '2.0.0' })), '--apply');
  const backups = path.join(ide.config, 'ktav-dev-backups');
  const versions = fs.readdirSync(backups).map((s) => fs.readdirSync(path.join(backups, s, 'previous', 'ktav-intellij', 'lib')).find((n) => n.endsWith('.jar')));
  assert.deepEqual(versions.sort(), ['ktav-intellij-0.1.0.jar', 'ktav-intellij-1.0.0.jar']);
});

test('fresh install into an IDE without Ktav', () => {
  const ide = makeIde({ ktav: 'none' });
  run('install', '--jetbrains', ide.plugins, '--plugin-zip', writeZip(ide, 'g.zip', pluginZip()), '--apply');
  assert.ok(fs.existsSync(path.join(ide.ktavDir, 'lib', 'ktav-intellij-9.9.9.jar')));
});

test('refusals leave the sandbox untouched', async (t) => {
  const cases = [
    ['running IDE (.lock present)', (ide) => fs.writeFileSync(path.join(ide.config, '.lock'), ''), /running/],
    ['foreign folder named ktav-intellij', null, /not the lang\.ktav plugin/, { ktav: 'foreign' }],
    ['target is not a plugins directory', null, /must be an IDE "plugins" directory/, {}, (ide) => ide.config],
    ['config directory missing', null, /does not exist/, {}, (ide) => path.join(ide.root, 'Nope', 'plugins')],
    ['symlinked ktav-intellij', (ide) => {
      const real = path.join(path.dirname(ide.root), 'elsewhere');
      fs.renameSync(ide.ktavDir, real);
      fs.symlinkSync(real, ide.ktavDir, process.platform === 'win32' ? 'junction' : 'dir');
    }, /symlink\/junction/],
  ];
  for (const [name, setup, error, opts = {}, targetOf] of cases) {
    await t.test(name, () => {
      const ide = makeIde(opts);
      if (setup) setup(ide);
      const zip = writeZip(ide, 'good.zip', pluginZip());
      const before = snapshot(ide.base);
      assert.throws(() => run('install', '--jetbrains', targetOf ? targetOf(ide) : ide.plugins, '--plugin-zip', zip, '--apply'), error);
      assert.deepEqual(snapshot(ide.base), before);
    });
  }
});

test('hostile or wrong archives are rejected before any write', async (t) => {
  const good = (name, content = 'x') => [name, content];
  const cases = [
    ['parent traversal', [good('ktav-intellij/../escape.txt')], /unsafe entry path/],
    ['sibling plugin', [good('other-plugin/lib/x.jar')], /outside ktav-intellij/],
    ['absolute path', [good('/etc/passwd')], /outside ktav-intellij/],
    ['backslash', [good('ktav-intellij/lib\\..\\..\\x')], /unsafe entry name/],
    ['drive letter', [good('ktav-intellij/C:/x')], /unsafe entry name/],
    ['symlink entry', [['ktav-intellij/lib/link', '/etc', 0o120777]], /symlink entry/],
    ['duplicate', [good('ktav-intellij/lib/a'), good('ktav-intellij/lib/A')], /duplicate entry/],
  ];
  for (const [name, extra, error] of cases) {
    await t.test(name, () => {
      const ide = makeIde();
      const zip = writeZip(ide, 'bad.zip', pluginZip({ extra }));
      const before = snapshot(ide.base);
      assert.throws(() => run('install', '--jetbrains', ide.plugins, '--plugin-zip', zip, '--apply'), error);
      assert.deepEqual(snapshot(ide.base), before);
    });
  }
  await t.test('wrong plugin id', () => {
    const ide = makeIde();
    const zip = writeZip(ide, 'other.zip', pluginZip({ id: 'com.example.other' }));
    const before = snapshot(ide.base);
    assert.throws(() => run('install', '--jetbrains', ide.plugins, '--plugin-zip', zip, '--apply'), /expected lang\.ktav/);
    assert.deepEqual(snapshot(ide.base), before);
  });
  await t.test('corrupt archive', () => {
    const ide = makeIde();
    const buf = pluginZip({ extra: [['ktav-intellij/lib/data.bin', 'CORRUPT-ME']] });
    buf[buf.indexOf('CORRUPT-ME')] ^= 0xff;
    const zip = writeZip(ide, 'corrupt.zip', buf);
    const before = snapshot(ide.base);
    assert.throws(() => run('install', '--jetbrains', ide.plugins, '--plugin-zip', zip, '--apply'), /zip:/);
    assert.deepEqual(snapshot(ide.base), before);
  });
});

test('IDE names resolve through the configured roots; the same IDE twice is refused', () => {
  const ide = makeIde();
  const zip = writeZip(ide, 'good.zip', pluginZip());
  const saved = process.env.KTAV_DEV_INSTALL_JETBRAINS_ROOTS;
  process.env.KTAV_DEV_INSTALL_JETBRAINS_ROOTS = ide.root;
  try {
    assert.match(run('list'), /FakeIDE2025\.1\s+Ktav 0\.1\.0/);
    assert.match(run('install', '--jetbrains', 'fakeide2025.1', '--plugin-zip', zip), /DRY RUN/);
    assert.throws(() => run('install', '--jetbrains', 'Missing2025.1', '--plugin-zip', zip), /not found/);
    assert.throws(() => run('install', '--jetbrains', 'FakeIDE2025.1', '--jetbrains', ide.plugins, '--plugin-zip', zip), /selected twice/);
  } finally {
    if (saved === undefined) delete process.env.KTAV_DEV_INSTALL_JETBRAINS_ROOTS;
    else process.env.KTAV_DEV_INSTALL_JETBRAINS_ROOTS = saved;
  }
});

test('VS Code targets go through the editor CLI only', () => {
  const dir = path.join(sandbox, 'fake-code');
  fs.mkdirSync(dir);
  const record = path.join(dir, 'calls.log');
  fs.writeFileSync(path.join(dir, 'fake-code.js'), `
    const fs = require('fs');
    fs.appendFileSync(${JSON.stringify(record)}, JSON.stringify(process.argv.slice(2)) + '\\n');
    if (process.argv.includes('--list-extensions')) console.log('other.ext@1.0.0\\nktav-lang.ktav@0.5.0');
  `);
  let cli;
  if (process.platform === 'win32') {
    cli = path.join(dir, 'fake-code.cmd');
    fs.writeFileSync(cli, `@"${process.execPath}" "%~dp0fake-code.js" %*\r\n`);
  } else {
    cli = path.join(dir, 'fake-code');
    fs.writeFileSync(cli, `#!/bin/sh\nexec "${process.execPath}" "$(dirname "$0")/fake-code.js" "$@"\n`, { mode: 0o755 });
  }
  const vsix = path.join(dir, 'ktav-9.9.9.vsix');
  fs.writeFileSync(vsix, 'vsix');
  assert.match(run('install', '--vscode', cli, '--vsix', vsix), /DRY RUN/);
  const dryCalls = fs.readFileSync(record, 'utf8').trim().split('\n').map((l) => JSON.parse(l));
  assert.ok(dryCalls.every((c) => c.includes('--list-extensions')), 'dry run only lists');
  run('install', '--vscode', cli, '--vsix', vsix, '--apply');
  const calls = fs.readFileSync(record, 'utf8').trim().split('\n').map((l) => JSON.parse(l));
  assert.deepEqual(calls.filter((c) => c.includes('--install-extension')), [['--install-extension', vsix, '--force']]);
});

test('source guard: no destructive filesystem calls outside fsguard.mjs', () => {
  const files = [path.join(SCRIPTS, 'dev-install.mjs'),
    ...fs.readdirSync(path.join(SCRIPTS, 'dev-install')).map((n) => path.join(SCRIPTS, 'dev-install', n))];
  const banned = /\b(rmSync|rm|unlinkSync|unlink|rmdirSync|rmdir|renameSync|rename|writeFileSync|truncateSync|rimraf)\s*\(|\bshell:\s*true|\bexecSync\b|(?<!\.)\bexec\s*\(/;
  const recursiveOutsideMkdir = (line) => /recursive:\s*true/.test(line) && !/mkdirSync\(/.test(line);
  for (const file of files) {
    const lines = fs.readFileSync(file, 'utf8').split('\n');
    lines.forEach((line, i) => {
      if (/^\s*\/\//.test(line)) return;
      if (path.basename(file) === 'fsguard.mjs') {
        assert.ok(!/\b(rmSync|unlinkSync|rimraf)\b/.test(line) && !recursiveOutsideMkdir(line), `${file}:${i + 1}: ${line}`);
        return;
      }
      assert.ok(!banned.test(line) && !recursiveOutsideMkdir(line), `${path.basename(file)}:${i + 1}: ${line.trim()}`);
    });
  }
});
