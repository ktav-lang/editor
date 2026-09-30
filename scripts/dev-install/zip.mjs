// Minimal read-only ZIP reader (stored + deflate, no ZIP64). It never
// writes anything; callers decide where validated entries may go.
import { inflateRawSync } from 'node:zlib';

const EOCD_SIG = 0x06054b50;
const CEN_SIG = 0x02014b50;
const LOC_SIG = 0x04034b50;
const MAX_TOTAL = 512 * 1024 * 1024;

const CRC_TABLE = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

export function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = CRC_TABLE[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function fail(msg) {
  throw new Error(`zip: ${msg}`);
}

/** Parse the central directory. Returns entries with a lazy `data()`. */
export function readZip(buf) {
  let eocd = -1;
  for (let i = buf.length - 22; i >= Math.max(0, buf.length - 22 - 0xffff); i--) {
    if (buf.readUInt32LE(i) === EOCD_SIG) { eocd = i; break; }
  }
  if (eocd < 0) fail('end of central directory not found');
  const count = buf.readUInt16LE(eocd + 10);
  const cdSize = buf.readUInt32LE(eocd + 12);
  const cdOffset = buf.readUInt32LE(eocd + 16);
  if (count === 0xffff || cdOffset === 0xffffffff) fail('ZIP64 is not supported');
  if (cdOffset + cdSize > eocd) fail('central directory out of bounds');

  const entries = [];
  let p = cdOffset;
  let total = 0;
  for (let i = 0; i < count; i++) {
    if (p + 46 > buf.length || buf.readUInt32LE(p) !== CEN_SIG) fail('bad central directory entry');
    const flags = buf.readUInt16LE(p + 8);
    const method = buf.readUInt16LE(p + 10);
    const crc = buf.readUInt32LE(p + 16);
    const csize = buf.readUInt32LE(p + 20);
    const usize = buf.readUInt32LE(p + 24);
    const nameLen = buf.readUInt16LE(p + 28);
    const extraLen = buf.readUInt16LE(p + 30);
    const commentLen = buf.readUInt16LE(p + 32);
    const madeBy = buf.readUInt16LE(p + 4) >> 8;
    const externalAttr = buf.readUInt32LE(p + 38);
    const localOffset = buf.readUInt32LE(p + 42);
    const name = buf.subarray(p + 46, p + 46 + nameLen).toString('utf8');
    if (flags & 0x1) fail(`encrypted entry: ${name}`);
    if (method !== 0 && method !== 8) fail(`unsupported compression ${method}: ${name}`);
    if (csize === 0xffffffff || usize === 0xffffffff || localOffset === 0xffffffff) fail('ZIP64 is not supported');
    total += usize;
    if (total > MAX_TOTAL) fail('archive expands beyond the size limit');
    const unixMode = madeBy === 3 ? (externalAttr >>> 16) & 0xffff : 0;
    entries.push({
      name,
      isDir: name.endsWith('/'),
      isSymlink: (unixMode & 0o170000) === 0o120000,
      mode: unixMode & 0o777,
      data() {
        if (buf.readUInt32LE(localOffset) !== LOC_SIG) fail(`bad local header: ${name}`);
        const start = localOffset + 30 + buf.readUInt16LE(localOffset + 26) + buf.readUInt16LE(localOffset + 28);
        if (start + csize > buf.length) fail(`entry out of bounds: ${name}`);
        const raw = buf.subarray(start, start + csize);
        const out = method === 0 ? Buffer.from(raw) : inflateRawSync(raw);
        if (out.length !== usize || crc32(out) !== crc) fail(`checksum mismatch: ${name}`);
        return out;
      },
    });
    p += 46 + nameLen + extraLen + commentLen;
  }
  return entries;
}

/**
 * Reject any entry that could land outside `<root>/`: absolute paths,
 * drive letters, backslashes, `.`/`..` segments, symlinks, duplicates.
 */
export function assertEntriesUnder(entries, root) {
  if (entries.length === 0) fail('empty archive');
  const seen = new Set();
  for (const e of entries) {
    const n = e.name;
    if (!n.startsWith(`${root}/`)) fail(`entry outside ${root}/: ${n}`);
    if (n.includes('\\') || n.includes(':') || n.includes('\0')) fail(`unsafe entry name: ${n}`);
    const parts = n.replace(/\/$/, '').split('/');
    if (parts.some((s) => s === '' || s === '.' || s === '..')) fail(`unsafe entry path: ${n}`);
    if (e.isSymlink) fail(`symlink entry: ${n}`);
    const key = n.replace(/\/$/, '').toLowerCase();
    if (seen.has(key)) fail(`duplicate entry: ${n}`);
    seen.add(key);
  }
}
