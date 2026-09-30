// Key spans and the paired quotes of quoted key segments (§ 5.3.3),
// derived from the server's semantic tokens. ktav-lsp emits every key
// segment as a `property` token; for a quoted segment the token covers
// only the content and carries the `quoted` modifier, so the delimiters
// sit right before and right after it. No VS Code API here.

export interface QuotePair {
  line: number;
  open: number;
  close: number;
}

export interface KeySpan {
  line: number;
  start: number;
  end: number;
}

export interface KeyTokens {
  keys: KeySpan[];
  pairs: QuotePair[];
}

export interface Legend {
  tokenTypes: readonly string[];
  tokenModifiers: readonly string[];
}

const QUOTES = new Set(['"', "'", "`"]);

/** Decode LSP semantic-token data (5-int groups) into key spans and quote pairs. */
export function decodeKeyTokens(
  data: readonly number[],
  legend: Legend,
  lineAt: (line: number) => string | undefined,
): KeyTokens {
  const property = legend.tokenTypes.indexOf("property");
  const bit = legend.tokenModifiers.indexOf("quoted");
  const quotedMask = bit < 0 ? 0 : 1 << bit;
  const keys: KeySpan[] = [];
  const pairs: QuotePair[] = [];
  let line = 0;
  let start = 0;
  for (let i = 0; i + 4 < data.length; i += 5) {
    const deltaLine = data[i];
    line += deltaLine;
    start = deltaLine === 0 ? start + data[i + 1] : data[i + 1];
    if (data[i + 3] !== property) continue;
    const end = start + data[i + 2];
    keys.push({ line, start, end });
    if ((data[i + 4] & quotedMask) === 0) continue;
    const text = lineAt(line);
    const q = text?.[start - 1];
    if (q !== undefined && QUOTES.has(q) && text?.[end] === q) pairs.push({ line, open: start - 1, close: end });
  }
  return { keys, pairs };
}

/**
 * The pair whose quote touches the cursor, like bracket matching: the
 * cursor sits right before or right after either quote.
 */
export function pairAtCursor(pairs: readonly QuotePair[], line: number, character: number): QuotePair | undefined {
  return pairs.find((p) => p.line === line
    && (character === p.open || character === p.open + 1 || character === p.close || character === p.close + 1));
}
