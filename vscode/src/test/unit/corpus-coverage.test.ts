// Cross-cutting coverage test: runs the shared TextMate grammar
// (../../../../grammars/ktav.tmLanguage.json) over the FULL Ktav 0.8.0
// conformance corpus (spec/versions/0.8/tests), not just the hand-picked
// vectors in grammar-tokens.test.ts. Two properties are checked for every
// `valid/**/<name>.ktav` fixture:
//
//   1. No token gets an `invalid.*` scope (nothing in `valid/` should be
//      flagged as illegal).
//   2. The number of tokens scoped `constant.numeric.*` /
//      `constant.language.boolean.*` / `constant.language.null.*`, summed
//      over the whole file, equals the number of number/boolean/null
//      leaves in the fixture's oracle (`<name>.json`) — i.e. the grammar
//      neither misses a literal nor over-highlights a string as one.
//
// `invalid/bad_escape/unicode_escape_*` fixtures are checked the other way:
// they are the known lower bound of `invalid.illegal.*` production.

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

const CORPUS_ROOT = path.join(
  __dirname,
  "..",
  "..",
  "..",
  "..",
  "spec",
  "versions",
  "0.8",
  "tests",
);
const VALID_ROOT = path.join(CORPUS_ROOT, "valid");
const INVALID_ROOT = path.join(CORPUS_ROOT, "invalid");

interface Tok {
  startIndex: number;
  endIndex: number;
  scopes: string[];
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

// Tokenizes a whole file, threading ruleStack across lines the way a real
// editor does. Strips a leading BOM/U+FEFF line artifact the same way
// vscode-textmate would receive it from an open document (raw text, BOM
// included as the first character of line 1 — the grammar is responsible
// for not choking on it, same as bom_boundary fixtures require).
function tokenizeFile(grammar: IGrammar, text: string): Tok[] {
  const lines = text.split(/\r\n|\r|\n/);
  let ruleStack = INITIAL;
  const all: Tok[] = [];
  for (const line of lines) {
    const result = grammar.tokenizeLine(line, ruleStack);
    for (const t of result.tokens) {
      all.push({ startIndex: t.startIndex, endIndex: t.endIndex, scopes: t.scopes });
    }
    ruleStack = result.ruleStack;
  }
  return all;
}

function hasScopePrefix(tok: Tok, prefix: string): boolean {
  return tok.scopes.some((s) => s.startsWith(prefix));
}

function countLeaves(value: unknown): { numbers: number; booleans: number; nulls: number } {
  if (value === null) {
    return { numbers: 0, booleans: 0, nulls: 1 };
  }
  if (typeof value === "boolean") {
    return { numbers: 0, booleans: 1, nulls: 0 };
  }
  if (typeof value === "number") {
    return { numbers: 1, booleans: 0, nulls: 0 };
  }
  if (typeof value === "string") {
    return { numbers: 0, booleans: 0, nulls: 0 };
  }
  const acc = { numbers: 0, booleans: 0, nulls: 0 };
  const children: unknown[] = Array.isArray(value) ? value : Object.values(value as object);
  for (const child of children) {
    const c = countLeaves(child);
    acc.numbers += c.numbers;
    acc.booleans += c.booleans;
    acc.nulls += c.nulls;
  }
  return acc;
}

function walk(dir: string, out: string[]): void {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(full, out);
    } else if (entry.name.endsWith(".ktav") && !entry.name.endsWith(".canonical.ktav")) {
      out.push(full);
    }
  }
}

function listValidFixtures(): string[] {
  const files: string[] = [];
  walk(VALID_ROOT, files);
  files.sort();
  return files;
}

// Numbers outside the i64/f64 domain, written WITHOUT a `::` raw marker.
// Per § 3.6/§ 5.2 the grammar classifies purely on lexical form — it has no
// notion of i64/f64 range — so it correctly scopes these as numeric. The
// *parser* is the layer that range-checks and falls back to String, which
// is why the oracle records them as JSON strings. Comparing token counts
// against the oracle for these five fixtures would fault the grammar for a
// domain check that isn't its job; they're excluded here instead of
// "fixed" by moving range-checking into the highlighter.
const DOMAIN_OVERFLOW_ALLOWLIST = new Set(
  [
    ["numbers", "integer", "big_overflow_to_string.ktav"],
    ["numbers", "integer", "i64_overflow_to_string.ktav"],
    ["numbers", "float", "just_above_max_finite_to_string.ktav"],
    ["numbers", "float", "negative_overflow_to_string.ktav"],
    ["numbers", "float", "positive_overflow_to_string.ktav"],
  ].map((parts) => path.join(VALID_ROOT, ...parts)),
);

suite("corpus coverage: grammar over spec/versions/0.8/tests/valid", () => {
  let grammar: IGrammar;
  const fixtures = listValidFixtures();

  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  test("corpus fixtures were actually found (guard against a silently empty run)", () => {
    assert.ok(
      fixtures.length > 100,
      `expected the full valid/ corpus to be present under ${VALID_ROOT}, found ${fixtures.length} fixture(s)`,
    );
  });

  for (const ktavPath of fixtures) {
    const rel = path.relative(VALID_ROOT, ktavPath);
    test(`${rel}: no invalid.* scope, and numeric/boolean/null token counts match the oracle`, () => {
      const text = fs.readFileSync(ktavPath, "utf8");
      const toks = tokenizeFile(grammar, text);

      for (const t of toks) {
        assert.ok(
          !hasScopePrefix(t, "invalid"),
          `unexpected invalid.* scope in ${rel}: ${JSON.stringify(t.scopes)}`,
        );
      }

      if (DOMAIN_OVERFLOW_ALLOWLIST.has(ktavPath)) {
        return;
      }

      const jsonPath = ktavPath.slice(0, -".ktav".length) + ".json";
      const oracle = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
      const expected = countLeaves(oracle);

      const numberTokens = toks.filter((t) => hasScopePrefix(t, "constant.numeric.")).length;
      const booleanTokens = toks.filter((t) => hasScopePrefix(t, "constant.language.boolean.")).length;
      const nullTokens = toks.filter((t) => hasScopePrefix(t, "constant.language.null.")).length;

      assert.strictEqual(
        numberTokens,
        expected.numbers,
        `${rel}: expected ${expected.numbers} constant.numeric.* token(s) per oracle, got ${numberTokens}`,
      );
      assert.strictEqual(
        booleanTokens,
        expected.booleans,
        `${rel}: expected ${expected.booleans} constant.language.boolean.* token(s) per oracle, got ${booleanTokens}`,
      );
      assert.strictEqual(
        nullTokens,
        expected.nulls,
        `${rel}: expected ${expected.nulls} constant.language.null.* token(s) per oracle, got ${nullTokens}`,
      );
    });
  }
});

suite("corpus coverage: invalid.illegal.escape.unicode.ktav lower bound", () => {
  let grammar: IGrammar;

  suiteSetup(async () => {
    grammar = await getGrammar();
  });

  // Known set of invalid/ fixtures that MUST produce invalid.illegal.*
  // (a lower bound — other invalid/ fixtures may be rejected by the parser
  // for reasons the grammar has no way to flag, e.g. duplicate keys).
  const KNOWN_ILLEGAL_ESCAPE_FIXTURES = [
    "bad_escape/unicode_escape_high_surrogate_then_non_surrogate.ktav",
    "bad_escape/unicode_escape_lone_high_surrogate.ktav",
    "bad_escape/unicode_escape_lone_low_surrogate.ktav",
    "bad_escape/unicode_escape_too_few_digits.ktav",
  ];

  for (const rel of KNOWN_ILLEGAL_ESCAPE_FIXTURES) {
    test(`${rel} produces invalid.illegal.escape.unicode.ktav`, () => {
      const text = fs.readFileSync(path.join(INVALID_ROOT, rel), "utf8");
      const toks = tokenizeFile(grammar, text);
      assert.ok(
        toks.some((t) => t.scopes.includes("invalid.illegal.escape.unicode.ktav")),
        `expected at least one invalid.illegal.escape.unicode.ktav token in ${rel}`,
      );
    });
  }
});
