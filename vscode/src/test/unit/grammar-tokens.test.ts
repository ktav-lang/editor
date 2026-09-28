// Tokenizer tests for the shared TextMate grammar (../../../../grammars/ktav.tmLanguage.json).
// Loads the real grammar through vscode-textmate + vscode-oniguruma (the same
// engine VS Code and the IntelliJ TextMate bundle use) and asserts scopes for
// the Ktav 0.8.0 number/keyword/escape/quoted-key conformance vectors, across
// three contexts: a whole-line pair value, an array item, and an inline value.

import * as assert from "assert";
import * as fs from "fs";
import * as path from "path";
import type { IGrammar, IOnigLib } from "vscode-textmate";
import { INITIAL, Registry } from "vscode-textmate";
import { loadWASM, OnigScanner, OnigString } from "vscode-oniguruma";

const GRAMMAR_PATH = path.join(
  __dirname,
  "..",
  "..",
  "..",
  "..",
  "grammars",
  "ktav.tmLanguage.json",
);

interface Tok {
  startIndex: number;
  endIndex: number;
  scopes: string[];
  text: string;
}

let grammarPromise: Promise<IGrammar> | undefined;

function getOnigLib(): Promise<IOnigLib> {
  const wasmPath = path.join(
    path.dirname(require.resolve("vscode-oniguruma/package.json")),
    "release",
    "onig.wasm",
  );
  const wasmBin = fs.readFileSync(wasmPath);
  const arrayBuffer = wasmBin.buffer.slice(
    wasmBin.byteOffset,
    wasmBin.byteOffset + wasmBin.byteLength,
  );
  return loadWASM(arrayBuffer).then(() => ({
    createOnigScanner(patterns: string[]) {
      return new OnigScanner(patterns);
    },
    createOnigString(s: string) {
      return new OnigString(s);
    },
  }));
}

function getGrammar(): Promise<IGrammar> {
  if (!grammarPromise) {
    const registry = new Registry({
      onigLib: getOnigLib(),
      loadGrammar: async (scopeName: string) => {
        if (scopeName === "source.ktav") {
          const content = fs.readFileSync(GRAMMAR_PATH, "utf8");
          return JSON.parse(content);
        }
        return null;
      },
    });
    grammarPromise = registry.loadGrammar("source.ktav").then((g) => {
      if (!g) {
        throw new Error("failed to load source.ktav grammar");
      }
      return g;
    });
  }
  return grammarPromise;
}

function tokenizeLine(grammar: IGrammar, line: string): Tok[] {
  const result = grammar.tokenizeLine(line, INITIAL);
  return result.tokens.map((t) => ({
    startIndex: t.startIndex,
    endIndex: t.endIndex,
    scopes: t.scopes,
    text: line.slice(t.startIndex, t.endIndex),
  }));
}

function tokensInRange(toks: Tok[], start: number, end: number): Tok[] {
  return toks.filter((t) => t.startIndex < end && t.endIndex > start);
}

// Threads ruleStack across lines, the way a real editor tokenizes a document
// line-by-line. Needed for any construct that only exists inside a stateful
// multi-line context (e.g. a genuine `array-item` line inside an open `[`).
function tokenizeLines(grammar: IGrammar, lines: string[]): Tok[][] {
  let ruleStack = INITIAL;
  const out: Tok[][] = [];
  for (const line of lines) {
    const result = grammar.tokenizeLine(line, ruleStack);
    out.push(
      result.tokens.map((t) => ({
        startIndex: t.startIndex,
        endIndex: t.endIndex,
        scopes: t.scopes,
        text: line.slice(t.startIndex, t.endIndex),
      })),
    );
    ruleStack = result.ruleStack;
  }
  return out;
}

// Asserts that `value` occupies exactly one token in `line` (found via the
// last occurrence, to dodge accidental earlier substring hits) and that its
// scope list contains `expectedScope`.
function checkScope(
  toks: Tok[],
  line: string,
  value: string,
  expectedScope: string,
): void {
  const idx = line.lastIndexOf(value);
  assert.notStrictEqual(idx, -1, `value ${JSON.stringify(value)} not found in ${JSON.stringify(line)}`);
  const span = tokensInRange(toks, idx, idx + value.length);
  assert.strictEqual(
    span.length,
    1,
    `expected exactly one token for ${JSON.stringify(value)} in ${JSON.stringify(line)}, got ${JSON.stringify(span)}`,
  );
  assert.strictEqual(span[0].startIndex, idx, `token start mismatch for ${JSON.stringify(value)} in ${JSON.stringify(line)}`);
  assert.strictEqual(span[0].endIndex, idx + value.length, `token end mismatch for ${JSON.stringify(value)} in ${JSON.stringify(line)}`);
  assert.ok(
    span[0].scopes.includes(expectedScope),
    `expected scope ${expectedScope} on ${JSON.stringify(value)} in ${JSON.stringify(line)}, got ${JSON.stringify(span[0].scopes)}`,
  );
}

function checkNotScope(
  toks: Tok[],
  line: string,
  value: string,
  forbiddenScope: string,
): void {
  const idx = line.lastIndexOf(value);
  assert.notStrictEqual(idx, -1, `value ${JSON.stringify(value)} not found in ${JSON.stringify(line)}`);
  const span = tokensInRange(toks, idx, idx + value.length);
  for (const t of span) {
    assert.ok(
      !t.scopes.includes(forbiddenScope),
      `did not expect scope ${forbiddenScope} on ${JSON.stringify(t.text)} (value ${JSON.stringify(value)}) in ${JSON.stringify(line)}, got ${JSON.stringify(t.scopes)}`,
    );
  }
}

function assertScope(grammar: IGrammar, line: string, value: string, expectedScope: string): void {
  checkScope(tokenizeLine(grammar, line), line, value, expectedScope);
}

function assertNotScope(grammar: IGrammar, line: string, value: string, forbiddenScope: string): void {
  checkNotScope(tokenizeLine(grammar, line), line, value, forbiddenScope);
}

// Variants for a value living on its own line inside a real, stateful
// multi-line array (`a: [` / value line / `]`), as opposed to an inline
// `a: [value]` or a whole-line pair value.
function assertArrayItemScope(grammar: IGrammar, value: string, expectedScope: string): void {
  const line = `  ${value}`;
  const perLine = tokenizeLines(grammar, ["a: [", line, "]"]);
  checkScope(perLine[1], line, value, expectedScope);
}

function assertArrayItemNotScope(grammar: IGrammar, value: string, forbiddenScope: string): void {
  const line = `  ${value}`;
  const perLine = tokenizeLines(grammar, ["a: [", line, "]"]);
  checkNotScope(perLine[1], line, value, forbiddenScope);
}

const STRING_SCOPE = "string.unquoted.ktav";
const RAW_STRING_SCOPE = "string.unquoted.raw.ktav";
const NUMBER_SCOPES = ["constant.numeric.integer.ktav", "constant.numeric.float.ktav"];
const ARRAY_NUMBER_SCOPE = "constant.numeric.ktav";

// § 3.6 / § 5.2 number vectors. `kind` says which numeric scope a value gets
// at pair/inline granularity (where integer and float are told apart).
const NUMBER_VECTORS: Array<{ value: string; kind: "integer" | "float" }> = [
  { value: "0", kind: "integer" },
  { value: "7", kind: "integer" },
  { value: "-7", kind: "integer" },
  { value: "+7", kind: "integer" },
  { value: "1_000", kind: "integer" },
  { value: "0x1A", kind: "integer" },
  { value: "0o755", kind: "integer" },
  { value: "0b1010", kind: "integer" },
  { value: "-0x1f", kind: "integer" },
  { value: "0x1_A", kind: "integer" },
  { value: "-0", kind: "integer" },
  { value: "0.5", kind: "float" },
  { value: "-0.5", kind: "float" },
  { value: "1e3", kind: "float" },
  { value: "1E3", kind: "float" },
  { value: "1e+3", kind: "float" },
  { value: "1_0.5_0", kind: "float" },
  { value: "6.022e23", kind: "float" },
  { value: "0e0", kind: "float" },
];

// Vectors that MUST fall through to String per § 3.6 / § 5.2.
const STRING_NUMBER_VECTORS = [
  "01234", "00", "0_7", "-045", "+007", "01.5", "05e3",
  "1_", "_1", "1__0", "0x", "0x_1", "0X1A", "0b102", "0o9",
  "1.", ".5", "1e", "1e+", "1.5e", "1_.5", "1._5", "1.2.3",
  "2026-09-28", "127.0.0.1", "-", "+",
];

suite("grammar: number literals — whole-line pair value", () => {
  let grammar: IGrammar;
  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  for (const { value, kind } of NUMBER_VECTORS) {
    test(`${JSON.stringify(value)} is ${kind}`, () => {
      const scope = kind === "integer" ? NUMBER_SCOPES[0] : NUMBER_SCOPES[1];
      assertScope(grammar, `a: ${value}`, value, scope);
    });
  }

  for (const value of STRING_NUMBER_VECTORS) {
    test(`${JSON.stringify(value)} is a String, not a number`, () => {
      assertNotScope(grammar, `a: ${value}`, value, NUMBER_SCOPES[0]);
      assertNotScope(grammar, `a: ${value}`, value, NUMBER_SCOPES[1]);
      assertScope(grammar, `a: ${value}`, value, STRING_SCOPE);
    });
  }
});

suite("grammar: number literals — array item", () => {
  let grammar: IGrammar;
  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  for (const { value } of NUMBER_VECTORS) {
    test(`${JSON.stringify(value)} is a number`, () => {
      assertArrayItemScope(grammar, value, ARRAY_NUMBER_SCOPE);
    });
  }

  for (const value of STRING_NUMBER_VECTORS) {
    test(`${JSON.stringify(value)} is a String, not a number`, () => {
      assertArrayItemNotScope(grammar, value, ARRAY_NUMBER_SCOPE);
      assertArrayItemScope(grammar, value, STRING_SCOPE);
    });
  }
});

suite("grammar: number literals — inline value", () => {
  let grammar: IGrammar;
  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  for (const { value, kind } of NUMBER_VECTORS) {
    test(`pair {k: ${value}} is ${kind}`, () => {
      const scope = kind === "integer" ? NUMBER_SCOPES[0] : NUMBER_SCOPES[1];
      assertScope(grammar, `a: {k: ${value}}`, value, scope);
    });
    test(`array item [${value}] is ${kind}`, () => {
      const scope = kind === "integer" ? NUMBER_SCOPES[0] : NUMBER_SCOPES[1];
      assertScope(grammar, `a: [${value}]`, value, scope);
    });
  }

  for (const value of STRING_NUMBER_VECTORS) {
    test(`pair {k: ${value}} is a String, not a number`, () => {
      assertNotScope(grammar, `a: {k: ${value}}`, value, NUMBER_SCOPES[0]);
      assertNotScope(grammar, `a: {k: ${value}}`, value, NUMBER_SCOPES[1]);
      assertScope(grammar, `a: {k: ${value}}`, value, STRING_SCOPE);
    });
  }
});

suite("grammar: keywords are exact-lowercase", () => {
  let grammar: IGrammar;
  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  test("true/false/null are keywords on a whole-line value", () => {
    assertScope(grammar, "a: true", "true", "constant.language.boolean.ktav");
    assertScope(grammar, "a: false", "false", "constant.language.boolean.ktav");
    assertScope(grammar, "a: null", "null", "constant.language.null.ktav");
  });

  test("True/NULL/False are Strings, not keywords", () => {
    assertNotScope(grammar, "a: True", "True", "constant.language.boolean.ktav");
    assertNotScope(grammar, "a: NULL", "NULL", "constant.language.null.ktav");
    assertNotScope(grammar, "a: False", "False", "constant.language.boolean.ktav");
  });

  test("keywords inside an inline pair", () => {
    assertScope(grammar, "a: {k: true}", "true", "constant.language.boolean.ktav");
    assertScope(grammar, "a: {k: null}", "null", "constant.language.null.ktav");
  });
});

suite("grammar: whitespace around values (§ 3.3 full 25-code-point set)", () => {
  let grammar: IGrammar;
  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  // A sample spanning ASCII, NBSP, EM SPACE and IDEOGRAPHIC SPACE.
  const samples = ["\u0009", " ", " ", "　", "\u0085"];

  for (const ws of samples) {
    test(`trailing U+${ws.codePointAt(0)!.toString(16).toUpperCase().padStart(4, "0")} does not break number classification`, () => {
      assertScope(grammar, `a: 7${ws}`, "7", "constant.numeric.integer.ktav");
    });
    test(`leading U+${ws.codePointAt(0)!.toString(16).toUpperCase().padStart(4, "0")} does not break number classification`, () => {
      assertScope(grammar, `a:${ws}7`, "7", "constant.numeric.integer.ktav");
    });
  }
});

suite("grammar: \\uXXXX unicode escapes (§ 3.7.1)", () => {
  let grammar: IGrammar;
  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  test("ordinary non-surrogate escape in a key is valid", () => {
    // key: Abc: v  -> escape covers A only
    assertScope(grammar, "\\u0041bc: v", "\\u0041", "constant.character.escape.unicode.ktav");
  });

  test("valid surrogate pair in a key tokenizes as ONE escape token", () => {
    const line = "\\uD83D\\uDE00: v";
    const toks = tokenizeLine(grammar, line);
    const idx = line.indexOf("\\uD83D");
    const pairText = "\\uD83D\\uDE00";
    const span = tokensInRange(toks, idx, idx + pairText.length);
    assert.strictEqual(span.length, 1, `expected one token for the surrogate pair, got ${JSON.stringify(span)}`);
    assert.strictEqual(span[0].text, pairText);
    assert.ok(span[0].scopes.includes("constant.character.escape.unicode.ktav"));
  });

  test("lone high surrogate in a key is invalid", () => {
    assertScope(grammar, "\\uD83Dbc: v", "\\uD83D", "invalid.illegal.escape.unicode.ktav");
  });

  test("lone low surrogate in a key is invalid", () => {
    assertScope(grammar, "\\uDE00bc: v", "\\uDE00", "invalid.illegal.escape.unicode.ktav");
  });

  test("code points immediately outside the surrogate range are valid on their own", () => {
    assertScope(grammar, "\\uD7FFx: v", "\\uD7FF", "constant.character.escape.unicode.ktav");
    assertScope(grammar, "\\uE000x: v", "\\uE000", "constant.character.escape.unicode.ktav");
  });

  test("valid non-surrogate escape inside an inline value", () => {
    assertScope(grammar, "a: {k: \\u0041bc}", "\\u0041", "constant.character.escape.unicode.ktav");
  });

  test("valid surrogate pair inside an inline value tokenizes as ONE escape token", () => {
    const line = "a: {k: \\uD83D\\uDE00}";
    const toks = tokenizeLine(grammar, line);
    const idx = line.indexOf("\\uD83D");
    const pairText = "\\uD83D\\uDE00";
    const span = tokensInRange(toks, idx, idx + pairText.length);
    assert.strictEqual(span.length, 1, `expected one token for the surrogate pair, got ${JSON.stringify(span)}`);
    assert.ok(span[0].scopes.includes("constant.character.escape.unicode.ktav"));
  });

  test("lone surrogate inside an inline value is invalid", () => {
    assertScope(grammar, "a: {k: \\uD83Dbc}", "\\uD83D", "invalid.illegal.escape.unicode.ktav");
  });
});

suite("grammar: quoted key segments (§ 5.3.3)", () => {
  let grammar: IGrammar;
  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  test("comma inside a double-quoted key segment does not split the key, inside an inline object", () => {
    const line = 'a: {"a,b": 1, \'x:y\': 2}';
    const toks = tokenizeLine(grammar, line);
    const idx = line.indexOf('"a,b"');
    const span = tokensInRange(toks, idx, idx + '"a,b"'.length);
    // The whole quoted segment must be ONE key token, not split at the comma.
    const spanningToken = span.find((t) => t.startIndex === idx && t.endIndex === idx + '"a,b"'.length);
    assert.ok(spanningToken, `expected a single token spanning "a,b" in ${JSON.stringify(line)}, got ${JSON.stringify(span)}`);
    assert.ok(spanningToken!.scopes.includes("string.quoted.double.key.ktav"));
  });

  test("colon inside a single-quoted key segment does not act as the pair separator, inside an inline object", () => {
    const line = 'a: {"a,b": 1, \'x:y\': 2}';
    const toks = tokenizeLine(grammar, line);
    const idx = line.indexOf("'x:y'");
    const span = tokensInRange(toks, idx, idx + "'x:y'".length);
    const spanningToken = span.find((t) => t.startIndex === idx && t.endIndex === idx + "'x:y'".length);
    assert.ok(spanningToken, `expected a single token spanning 'x:y' in ${JSON.stringify(line)}, got ${JSON.stringify(span)}`);
    assert.ok(spanningToken!.scopes.includes("string.quoted.single.key.ktav"));
    // The value 2 must still be recognised as a number (i.e. the pair after
    // 'x:y' was correctly parsed as key/value, not swallowed by the quote).
    assertScope(grammar, line, "2", "constant.numeric.integer.ktav");
  });

  test("dotted key with a quoted middle segment: a.\"b.c\".d", () => {
    const line = 'a."b.c".d: 1';
    const toks = tokenizeLine(grammar, line);
    const quoted = '"b.c"';
    const idx = line.indexOf(quoted);
    const span = tokensInRange(toks, idx, idx + quoted.length);
    const spanningToken = span.find((t) => t.startIndex === idx && t.endIndex === idx + quoted.length);
    assert.ok(spanningToken, `expected a single token spanning "b.c" in ${JSON.stringify(line)}, got ${JSON.stringify(span)}`);
    assert.ok(spanningToken!.scopes.includes("string.quoted.double.key.ktav"));
    assertScope(grammar, line, "1", "constant.numeric.integer.ktav");
  });
});

suite("grammar: :: raw marker inside inline objects (§ 5.8.2)", () => {
  let grammar: IGrammar;
  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  test("value after :: is a raw String even when it looks numeric", () => {
    assertNotScope(grammar, "a: {k:: 7}", "7", "constant.numeric.integer.ktav");
    assertScope(grammar, "a: {k:: 7}", "7", RAW_STRING_SCOPE);
  });

  test("value after :: is a raw String even when it looks like a keyword", () => {
    assertNotScope(grammar, "a: {k:: true}", "true", "constant.language.boolean.ktav");
    assertScope(grammar, "a: {k:: true}", "true", RAW_STRING_SCOPE);
  });

  test("value after plain : is still type-inferred", () => {
    assertScope(grammar, "a: {k: 7}", "7", "constant.numeric.integer.ktav");
  });

  test("raw marker itself keeps its own scope", () => {
    const line = "a: {k:: 7}";
    assertScope(grammar, line, "::", "keyword.operator.marker.raw.ktav");
  });
});
