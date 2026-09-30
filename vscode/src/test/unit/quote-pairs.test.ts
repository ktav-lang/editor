import * as assert from "assert";
import { decodeKeyTokens, pairAtCursor } from "../../quote-pairs";

// Semantic-token groups: deltaLine, deltaStart, length, type, modifiers.
const LEGEND = { tokenTypes: ["comment", "keyword", "number", "string", "property"], tokenModifiers: ["quoted"] };

suite("quoted-key quote pairs", () => {
  const lines = ['"ключ в кавычках": да', "a.'b.c'.`d`: 1", 'plain: "not a key"'];
  const lineAt = (n: number) => lines[n];
  // Line 0: content 1..15 quoted; line 1: `a` bare, 'b.c' at 3, `d` at 9;
  // line 2: bare key plus a string value, no quoted modifier.
  const data = [
    0, 1, 15, 4, 1,
    1, 0, 1, 4, 0, 0, 3, 3, 4, 1, 0, 6, 1, 4, 1,
    1, 0, 5, 4, 0, 0, 7, 11, 3, 0,
  ];
  const { keys, pairs } = decodeKeyTokens(data, LEGEND, lineAt);

  test("pairs sit around quoted content only", () => {
    assert.deepStrictEqual(pairs, [
      { line: 0, open: 0, close: 16 },
      { line: 1, open: 2, close: 6 },
      { line: 1, open: 8, close: 10 },
    ]);
  });

  test("every key token is a key span; quoted spans exclude their quotes", () => {
    assert.deepStrictEqual(keys, [
      { line: 0, start: 1, end: 16 },
      { line: 1, start: 0, end: 1 }, { line: 1, start: 3, end: 6 }, { line: 1, start: 9, end: 10 },
      { line: 2, start: 0, end: 5 },
    ]);
  });

  test("cursor before or after either quote selects its pair, like bracket matching", () => {
    for (const ch of [0, 1, 16, 17]) {
      assert.deepStrictEqual(pairAtCursor(pairs, 0, ch), pairs[0], `column ${ch}`);
    }
    assert.strictEqual(pairAtCursor(pairs, 0, 5), undefined);
    assert.deepStrictEqual(pairAtCursor(pairs, 1, 11), pairs[2]);
    assert.strictEqual(pairAtCursor(pairs, 2, 7), undefined);
  });

  test("no quoted modifier in the legend, or mismatched delimiters, yields nothing", () => {
    assert.deepStrictEqual(decodeKeyTokens(data, { ...LEGEND, tokenModifiers: [] }, lineAt).pairs, []);
    assert.deepStrictEqual(decodeKeyTokens([0, 1, 3, 4, 1], LEGEND, () => '"abc\'').pairs, []);
  });
});
