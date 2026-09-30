package lang.ktav.highlighting

import com.intellij.psi.tree.IElementType
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Pure unit tests for [KtavLexer] — no IntelliJ runtime needed.
 * Values: numbers (§ 3.6/§ 5.2), keywords, comments, inline compounds and raw `::` values.
 */
class KtavLexerValuesTest {

    @Test
    fun number_value_inferred_from_form() {
        val toks = lexTokens("port: 8080\n")
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.INT_VALUE, toks[2].second)
        assertEquals("8080", toks[2].first)
    }

    @Test
    fun hex_integer_value() {
        val toks = lexTokens("mask: 0xFF_00\n")
        assertEquals(KtavTokenTypes.INT_VALUE, toks[2].second)
    }

    @Test
    fun float_value() {
        val toks = lexTokens("ratio: 1.5e3\n")
        assertEquals(KtavTokenTypes.FLOAT_VALUE, toks[2].second)
    }

    @Test
    fun double_hash_is_comment_single_hash_is_content() {
        val toks = lexTokens("## a comment\n")
        assertEquals(KtavTokenTypes.COMMENT, toks[0].second)
        assertEquals("## a comment", toks[0].first)
        // Single `#` at line start is NOT a comment (0.5.0).
        val t2 = lexTokens("#notacomment\n")
        assertEquals(false, t2[0].second == KtavTokenTypes.COMMENT)
    }

    @Test
    fun inline_object_brackets_are_brace_tokens() {
        val toks = lexTokens("a: {name: alice}\n")
        // a : { name : alice }
        assertEquals(KtavTokenTypes.KEY, toks[0].second)       // a
        assertEquals(KtavTokenTypes.COLON, toks[1].second)     // :
        assertEquals(KtavTokenTypes.LBRACE, toks[2].second)    // {
        assertEquals(KtavTokenTypes.KEY, toks[3].second)       // name
        assertEquals(KtavTokenTypes.COLON, toks[4].second)     // :
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[5].second) // alice
        assertEquals(KtavTokenTypes.RBRACE, toks[6].second)    // }
    }

    @Test
    fun inline_array_of_objects_nested_brackets() {
        val toks = lexTokens("xs: [{a: 1}, {b: 2}]\n")
        val types = toks.map { it.second }
        // Must contain balanced bracket tokens for matching.
        assertEquals(KtavTokenTypes.LBRACKET, types[2])
        assertEquals(KtavTokenTypes.LBRACE, types[3])
        assertEquals(KtavTokenTypes.RBRACE, types[7])   // a:1}
        assertEquals(KtavTokenTypes.COMMA, types[8])
        assertEquals(KtavTokenTypes.LBRACE, types[9])
        assertEquals(KtavTokenTypes.RBRACE, types[13])
        assertEquals(KtavTokenTypes.RBRACKET, types[14])
    }

    @Test
    fun inline_value_with_colon_stays_one_string() {
        // `C:/path` after a key inside an inline object is one value, not split.
        val toks = lexTokens("a: {win: C:/Users/x}\n")
        // a : { win : C:/Users/x }
        assertEquals(KtavTokenTypes.KEY, toks[3].second)         // win
        assertEquals(KtavTokenTypes.COLON, toks[4].second)       // :
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[5].second)
        assertEquals("C:/Users/x", toks[5].first)
        assertEquals(KtavTokenTypes.RBRACE, toks[6].second)
    }

    @Test
    fun literal_brace_in_top_level_scalar_is_not_structural() {
        // `hello{world` as a plain value is one string (not an inline object).
        val toks = lexTokens("note: hello{world\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("hello{world", toks[2].first)
    }

    @Test
    fun raw_marker_does_not_recognise_keywords() {
        val toks = lexTokens("flag:: true\n")
        assertEquals(KtavTokenTypes.DOUBLE_COLON, toks[1].second)
        // `true` after `::` must NOT be BOOLEAN
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("true", toks[2].first)
    }

    @Test
    fun array_item_keyword_recognized() {
        val toks = lexTokens("true\n")
        // `true` on a line with no `:` → keyword, not key
        assertEquals(KtavTokenTypes.BOOLEAN, toks[0].second)
    }

    @Test
    fun ip_address_value_is_string_not_float() {
        // Multiple dots ⇒ not a well-formed number; `127.0.0.1` is a string.
        val toks = lexTokens("host: 127.0.0.1\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("127.0.0.1", toks[2].first)
    }

    @Test
    fun version_value_is_string_not_float() {
        val toks = lexTokens("ver: 1.2.3\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("1.2.3", toks[2].first)
    }

    @Test
    fun single_dot_float_still_recognized() {
        val toks = lexTokens("min_version: 1.3\n")
        assertEquals(KtavTokenTypes.FLOAT_VALUE, toks[2].second)
    }

    @Test
    fun line_starting_with_brace_is_inline_object_array_item() {
        // An inline object as a bare array item must tokenize structurally,
        // not be mis-read as key `{name` + one opaque string value.
        val toks = lexTokens("{name: alice, age: 30}\n")
        val types = toks.map { it.second }
        assertEquals(KtavTokenTypes.LBRACE, types[0])        // {
        assertEquals(KtavTokenTypes.KEY, types[1])           // name
        assertEquals(KtavTokenTypes.COLON, types[2])         // :
        assertEquals(KtavTokenTypes.STRING_VALUE, types[3])  // alice
        assertEquals(KtavTokenTypes.COMMA, types[4])         // ,
        assertEquals(KtavTokenTypes.KEY, types[5])           // age
        assertEquals(KtavTokenTypes.COLON, types[6])         // :
        assertEquals(KtavTokenTypes.INT_VALUE, types[7])     // 30
        assertEquals(KtavTokenTypes.RBRACE, types[8])        // }
    }

    @Test
    fun line_starting_with_bracket_is_inline_array_item() {
        val toks = lexTokens("[1, 2, 3]\n")
        val types = toks.map { it.second }
        assertEquals(KtavTokenTypes.LBRACKET, types[0])
        assertEquals(KtavTokenTypes.INT_VALUE, types[1])     // 1
        assertEquals(KtavTokenTypes.COMMA, types[2])
        assertEquals(KtavTokenTypes.RBRACKET, types[6])
    }

    @Test
    fun bare_array_item_with_comma_is_one_string_no_bad_char() {
        // Inside a block array, `a, b` is ONE string item — the comma is
        // literal content (it separates items only inside an inline [a, b]).
        val toks = lexTokens("b: [\n    a, b\n    c, d\n]\n")
        assertEquals(true, toks.any { it.first == "a, b" && it.second == KtavTokenTypes.STRING_VALUE })
        assertEquals(true, toks.any { it.first == "c, d" && it.second == KtavTokenTypes.STRING_VALUE })
        // The comma must NOT be flagged as a bad character.
        assertEquals(false, toks.any { it.second == KtavTokenTypes.BAD_CHARACTER })
    }

    @Test
    fun bare_array_item_with_brackets_is_one_string() {
        // Mid-line `{` / `[` in a bare item are literal content, not structural,
        // so the string highlight isn't split.
        val toks = lexTokens("arr: [\n    hello{world\n    mid[bracket\n    plain\n]\n")
        assertEquals(true, toks.any { it.first == "hello{world" && it.second == KtavTokenTypes.STRING_VALUE })
        assertEquals(true, toks.any { it.first == "mid[bracket" && it.second == KtavTokenTypes.STRING_VALUE })
        assertEquals(false, toks.any { it.second == KtavTokenTypes.BAD_CHARACTER })
    }

    @Test
    fun bare_dotted_run_is_string_not_dotted_key() {
        // A bare line with dots but no `:` is a string array item (e.g. an IP),
        // not a dotted key.
        val toks = lexTokens("1.2.3.4\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[0].second)
        assertEquals("1.2.3.4", toks[0].first)
    }

    // ---------------------------------------------------------------
    // Spec 0.8.0: exact § 3.6 number grammar / § 5.2 redundant-zero rule
    // ---------------------------------------------------------------

    private fun classify(body: String): IElementType {
        val toks = lexTokens("v: $body\n")
        assertEquals("expected exactly one value token for body <$body>", 3, toks.size)
        assertEquals("v", toks[0].first)
        return toks[2].second
    }

    @Test
    fun exact_grammar_integers() {
        val cases = listOf("0", "7", "-7", "+7", "1_000", "0x1A", "0o755", "0b1010", "-0x1f", "0x1_A", "-0")
        for (c in cases) assertEquals("`$c` should be INT_VALUE", KtavTokenTypes.INT_VALUE, classify(c))
    }

    @Test
    fun exact_grammar_floats() {
        val cases = listOf("0.5", "-0.5", "1e3", "1E3", "1e+3", "1_0.5_0", "6.022e23", "0e0")
        for (c in cases) assertEquals("`$c` should be FLOAT_VALUE", KtavTokenTypes.FLOAT_VALUE, classify(c))
    }

    @Test
    fun exact_grammar_rejects_fall_through_to_string() {
        val cases = listOf(
            "01234", "00", "0_7", "-045", "+007", "01.5", "05e3",
            "1_", "_1", "1__0", "0x", "0x_1", "0X1A", "0b102", "0o9",
            "1.", ".5", "1e", "1e+", "1.5e", "1_.5", "1._5", "1.2.3",
            "2026-09-28", "127.0.0.1", "-", "+", "True", "NULL",
        )
        for (c in cases) assertEquals("`$c` should be STRING_VALUE", KtavTokenTypes.STRING_VALUE, classify(c))
    }

    @Test
    fun non_ascii_digits_are_not_numeric() {
        // Arabic-Indic digits ١٢٣ — Char.isDigit() would wrongly accept these.
        assertEquals(KtavTokenTypes.STRING_VALUE, classify("\u0661\u0662\u0663"))
    }

    @Test
    fun i64_overflow_stays_numeric_for_highlighting() {
        // Highlighting only infers from the § 3.6 grammar; it does not
        // range-check against i64.
        assertEquals(KtavTokenTypes.INT_VALUE, classify("99999999999999999999999999"))
    }

    @Test
    fun nbsp_and_ideographic_space_around_keyword_are_whitespace() {
        // U+00A0 NBSP and U+3000 ideographic space are § 3.3 whitespace,
        // not key/value content — Kotlin's default trim()/isWhitespace()
        // handle one but not the other; the explicit predicate must handle
        // both. Leading whitespace becomes its own WHITESPACE token (so the
        // value token's text is exactly "true"); trailing whitespace stays
        // inside the value token's span through EOL — same as a plain
        // trailing space always has — but is trimmed before classification,
        // so the type is still correctly inferred.
        val nbsp = lexTokens("a:\u00A0true\n")
        assertEquals(KtavTokenTypes.BOOLEAN, nbsp[2].second)
        assertEquals("true", nbsp[2].first)

        val ideographic = lexTokens("a:\u3000true\n")
        assertEquals(KtavTokenTypes.BOOLEAN, ideographic[2].second)
        assertEquals("true", ideographic[2].first)

        // Trailing whitespace doesn't defeat classification either.
        assertEquals(KtavTokenTypes.BOOLEAN, lexTokens("a: true\u00A0\n")[2].second)
        assertEquals(KtavTokenTypes.BOOLEAN, lexTokens("a: true\u3000\n")[2].second)
    }

    // ---------------------------------------------------------------
    // Spec 0.8.0: raw (`::`) values always String; escape forces String
    // ---------------------------------------------------------------

    @Test
    fun raw_value_inside_inline_object_is_always_string() {
        val toks = lexTokens("{r:: 42}\n")
        assertEquals(KtavTokenTypes.LBRACE, toks[0].second)
        assertEquals(KtavTokenTypes.KEY, toks[1].second)
        assertEquals(KtavTokenTypes.DOUBLE_COLON, toks[2].second)
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[3].second)
        assertEquals("42", toks[3].first)
        assertEquals(KtavTokenTypes.RBRACE, toks[4].second)
    }

    @Test
    fun raw_value_leading_brace_is_literal_not_nested_compound() {
        // Per § 5.8.5, `::`'s value never dispatches to nested-compound
        // parsing: a leading `{` is literal text. The raw scalar still
        // terminates at the first unescaped `,`/`}`/`]` (here the comma),
        // so `{open` is the literal String value, not an (unterminated)
        // nested object.
        val toks = lexTokens("{r:: {open, b: 2}\n")
        assertEquals(KtavTokenTypes.LBRACE, toks[0].second)   // outer {
        assertEquals(KtavTokenTypes.KEY, toks[1].second)      // r
        assertEquals(KtavTokenTypes.DOUBLE_COLON, toks[2].second)
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[3].second)
        assertEquals("{open", toks[3].first)
        assertEquals(KtavTokenTypes.COMMA, toks[4].second)
        assertEquals(KtavTokenTypes.KEY, toks[5].second)      // b
        assertEquals("b", toks[5].first)
        assertEquals(KtavTokenTypes.COLON, toks[6].second)
        assertEquals(KtavTokenTypes.INT_VALUE, toks[7].second)
        assertEquals(KtavTokenTypes.RBRACE, toks[8].second)   // outer }
    }

    @Test
    fun inline_value_with_recognised_escape_is_string() {
        // A recognised § 3.7 escape anywhere in an inline scalar body
        // forces String (§ 5.2), independent of what it looks like.
        val toks = lexTokens("{a: \\u0031}\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[3].second)
    }
}
