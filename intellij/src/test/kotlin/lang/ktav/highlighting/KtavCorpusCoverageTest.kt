package lang.ktav.highlighting

import com.google.gson.JsonElement
import com.google.gson.JsonParser
import com.intellij.psi.tree.IElementType
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File

/**
 * Cross-cutting coverage: runs [KtavLexer] over the FULL Ktav 0.8.0
 * conformance corpus (`spec/versions/0.8/tests/valid`), not just the
 * hand-picked vectors in [KtavLexerTest]. For every `<name>.ktav` fixture:
 *   - the lexer must not throw;
 *   - tokens must tile the whole text with no gap/overlap (and no
 *     [KtavTokenTypes.BAD_CHARACTER]);
 *   - the number of INT_VALUE+FLOAT_VALUE / BOOLEAN / NULL tokens must
 *     equal the number of number/boolean/null leaves in the fixture's
 *     oracle (`<name>.json`).
 *
 * Guard, not skip: a missing spec submodule fails loudly, mirroring
 * `lsp/tests/spec_conformance.rs`'s `spec_tests_dir()` — CI can never go
 * green on an uninitialised submodule.
 */
class KtavCorpusCoverageTest {

    private val corpusRoot: File by lazy {
        File(System.getProperty("user.dir"), "../spec/versions/0.8/tests").canonicalFile
    }

    private val validRoot: File by lazy { File(corpusRoot, "valid") }

    // Numbers outside the i64/f64 domain, written WITHOUT a `::` raw
    // marker. The lexer classifies purely on lexical form (§ 3.6/§ 5.2) —
    // domain range-checking is the parser's job — so it correctly keeps
    // lexing these as numeric, while the oracle records them as JSON
    // strings (the parser range-checked and fell back). Same 5-fixture
    // allowlist as vscode/src/test/unit/corpus-coverage.test.ts and
    // lsp/tests/spec_conformance.rs's SEMANTIC_TOKEN_LEAF_COUNT_EXCEPTIONS.
    private val domainOverflowExceptions = setOf(
        "numbers/integer/big_overflow_to_string.ktav",
        "numbers/integer/i64_overflow_to_string.ktav",
        "numbers/float/just_above_max_finite_to_string.ktav",
        "numbers/float/negative_overflow_to_string.ktav",
        "numbers/float/positive_overflow_to_string.ktav",
    )

    private fun listValidFixtures(): List<File> {
        val out = mutableListOf<File>()
        fun walk(dir: File) {
            for (f in dir.listFiles()!!.sortedBy { it.name }) {
                if (f.isDirectory) {
                    walk(f)
                } else if (f.name.endsWith(".ktav") && !f.name.endsWith(".canonical.ktav")) {
                    out += f
                }
            }
        }
        walk(validRoot)
        return out
    }

    private data class LeafCounts(val numbers: Int, val booleans: Int, val nulls: Int) {
        operator fun plus(o: LeafCounts) = LeafCounts(numbers + o.numbers, booleans + o.booleans, nulls + o.nulls)
    }

    private val zeroLeaves = LeafCounts(0, 0, 0)

    private fun countLeaves(e: JsonElement): LeafCounts = when {
        e.isJsonNull -> LeafCounts(0, 0, 1)
        e.isJsonPrimitive -> {
            val p = e.asJsonPrimitive
            when {
                p.isBoolean -> LeafCounts(0, 1, 0)
                p.isNumber -> LeafCounts(1, 0, 0)
                else -> zeroLeaves // string
            }
        }
        e.isJsonArray -> e.asJsonArray.fold(zeroLeaves) { acc, c -> acc + countLeaves(c) }
        e.isJsonObject -> e.asJsonObject.entrySet().fold(zeroLeaves) { acc, entry -> acc + countLeaves(entry.value) }
        else -> zeroLeaves
    }

    private data class Tok(val start: Int, val end: Int, val type: IElementType)

    private fun lexAll(text: String): List<Tok> {
        val lex = KtavLexer()
        lex.start(text, 0, text.length, 0)
        val out = mutableListOf<Tok>()
        while (lex.tokenType != null) {
            out += Tok(lex.tokenStart, lex.tokenEnd, lex.tokenType!!)
            lex.advance()
        }
        return out
    }

    @Test
    fun spec_submodule_is_initialised() {
        assertTrue(
            "spec submodule not initialised at $corpusRoot — run " +
                "`git submodule update --init --recursive`. This test fails " +
                "rather than skips so a missing corpus can never pass as green.",
            validRoot.isDirectory && File(corpusRoot, "invalid").isDirectory,
        )
    }

    @Test
    fun corpus_fixtures_were_found() {
        val fixtures = listValidFixtures()
        assertTrue(
            "expected the full valid/ corpus under $validRoot, found ${fixtures.size} fixture(s)",
            fixtures.size > 100,
        )
    }

    @Test
    fun lexer_covers_every_valid_fixture_without_gaps_and_matches_oracle_leaf_counts() {
        val fixtures = listValidFixtures()
        assertTrue("no valid fixtures found under $validRoot", fixtures.isNotEmpty())

        val failures = mutableListOf<String>()
        for (file in fixtures) {
            val rel = file.relativeTo(validRoot).path.replace('\\', '/')
            val text = file.readText(Charsets.UTF_8)

            val toks = try {
                lexAll(text)
            } catch (t: Throwable) {
                failures += "$rel: lexer threw $t"
                continue
            }

            var cursor = 0
            var tiled = true
            for (t in toks) {
                if (t.start != cursor) {
                    failures += "$rel: gap/overlap at offset $cursor (next token starts at ${t.start})"
                    tiled = false
                    break
                }
                if (t.type == KtavTokenTypes.BAD_CHARACTER) {
                    failures += "$rel: unexpected BAD_CHARACTER at [${t.start}, ${t.end})"
                    tiled = false
                    break
                }
                cursor = t.end
            }
            if (tiled && cursor != text.length) {
                failures += "$rel: tokens end at $cursor, expected ${text.length}"
            }

            if (rel in domainOverflowExceptions) continue

            val jsonFile = File(file.parentFile, file.name.removeSuffix(".ktav") + ".json")
            val oracle = JsonParser.parseString(jsonFile.readText(Charsets.UTF_8))
            val expected = countLeaves(oracle)

            val numberTokens = toks.count { it.type == KtavTokenTypes.INT_VALUE || it.type == KtavTokenTypes.FLOAT_VALUE }
            val keywordTokens = toks.count { it.type == KtavTokenTypes.BOOLEAN }
            val nullTokens = toks.count { it.type == KtavTokenTypes.NULL }

            if (numberTokens != expected.numbers) {
                failures += "$rel: number tokens=$numberTokens, oracle leaves=${expected.numbers}"
            }
            if (keywordTokens != expected.booleans) {
                failures += "$rel: keyword tokens=$keywordTokens, oracle leaves=${expected.booleans}"
            }
            if (nullTokens != expected.nulls) {
                failures += "$rel: null tokens=$nullTokens, oracle leaves=${expected.nulls}"
            }
        }
        assertTrue("${failures.size} fixture(s) failed:\n${failures.joinToString("\n")}", failures.isEmpty())
    }
}
