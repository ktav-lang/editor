package lang.ktav.highlighting

import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Pure unit tests for [KtavLexer] — no IntelliJ runtime needed.
 */
class KtavLexerTest {

    /** Run lexer over [text]; return list of (text, token) pairs (whitespace skipped). */
    private fun tokens(text: String): List<Pair<String, IElementType>> {
        val lex = KtavLexer()
        lex.start(text, 0, text.length, 0)
        val result = mutableListOf<Pair<String, IElementType>>()
        while (lex.tokenType != null) {
            val t = lex.tokenType ?: break
            if (t != TokenType.WHITE_SPACE) {
                result += text.substring(lex.tokenStart, lex.tokenEnd) to t
            }
            lex.advance()
        }
        return result
    }

    @Test
    fun cyrillic_key_is_KEY_token() {
        val toks = tokens("имя: Иван\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("имя", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("Иван", toks[2].first)
    }

    @Test
    fun ascii_key_is_KEY_token() {
        val toks = tokens("name: John\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("name", toks[0].first)
    }

    @Test
    fun lone_cr_separates_records_and_crlf_is_one_terminator() {
        val text = "a: 1\rb: true\r\nc: false"
        val lex = KtavLexer()
        lex.start(text, 0, text.length, 0)
        val all = mutableListOf<Pair<String, IElementType>>()
        while (lex.tokenType != null) {
            all += text.substring(lex.tokenStart, lex.tokenEnd) to lex.tokenType!!
            lex.advance()
        }

        assertEquals(
            listOf(
                "a" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                " " to TokenType.WHITE_SPACE,
                "1" to KtavTokenTypes.INT_VALUE,
                "\r" to TokenType.WHITE_SPACE,
                "b" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                " " to TokenType.WHITE_SPACE,
                "true" to KtavTokenTypes.BOOLEAN,
                "\r\n" to TokenType.WHITE_SPACE,
                "c" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                " " to TokenType.WHITE_SPACE,
                "false" to KtavTokenTypes.BOOLEAN,
            ),
            all,
        )
    }

    @Test
    fun quoted_keys_stop_at_cr_and_next_line_is_scanned_independently() {
        val toks = tokens("\"unterminated\r\"b\": true\r'c.d': false")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[0].second)
        assertEquals("\"unterminated", toks[0].first)
        assertEquals(KtavTokenTypes.KEY, toks[1].second)
        assertEquals("\"b\"", toks[1].first)
        assertEquals(KtavTokenTypes.BOOLEAN, toks[3].second)
        assertEquals(KtavTokenTypes.KEY, toks[4].second)
        assertEquals("'c.d'", toks[4].first)
        assertEquals(KtavTokenTypes.BOOLEAN, toks[6].second)
    }

    @Test
    fun line_endings_stop_quoted_keys_inline_values_and_comments() {
        for (ending in listOf("\n", "\r", "\r\n")) {
            assertEquals(
                "quoted key followed by $ending",
                listOf(
                    "\"a:b\"" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON,
                    "true" to KtavTokenTypes.BOOLEAN,
                    "next" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON,
                    "null" to KtavTokenTypes.NULL,
                ),
                tokens("\"a:b\": true${ending}next: null"),
            )
            assertEquals(
                "inline value followed by $ending",
                listOf(
                    "{" to KtavTokenTypes.LBRACE,
                    "v" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON,
                    "foo\\" to KtavTokenTypes.STRING_VALUE,
                    "next" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON,
                    "true" to KtavTokenTypes.BOOLEAN,
                ),
                tokens("{v: foo\\${ending}next: true"),
            )
            assertEquals(
                "comment followed by $ending",
                listOf(
                    "## note" to KtavTokenTypes.COMMENT,
                    "next" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON,
                    "true" to KtavTokenTypes.BOOLEAN,
                ),
                tokens("## note${ending}next: true"),
            )
        }
    }

    @Test
    fun backslash_cannot_escape_a_line_ending_in_a_key_or_quote() {
        for (ending in listOf("\n", "\r", "\r\n")) {
            assertEquals(
                listOf(
                    "name\\" to KtavTokenTypes.STRING_VALUE,
                    "next" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON,
                    "true" to KtavTokenTypes.BOOLEAN,
                ),
                tokens("name\\${ending}next: true"),
            )
            assertEquals(
                listOf(
                    "\"open\\" to KtavTokenTypes.STRING_VALUE,
                    "next" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON,
                    "true" to KtavTokenTypes.BOOLEAN,
                ),
                tokens("\"open\\${ending}next: true"),
            )
        }
    }

    @Test
    fun multiline_bodies_preserve_state_across_cr_and_match_their_closer() {
        val stripped = tokens("a: 1\rb: (\rbody\r)\rkey: true")
        assertEquals(
            listOf(
                "a" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "1" to KtavTokenTypes.INT_VALUE,
                "b" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "(" to KtavTokenTypes.MULTILINE_OPEN,
                "body" to KtavTokenTypes.MULTILINE_TEXT,
                ")" to KtavTokenTypes.MULTILINE_CLOSE,
                "key" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "true" to KtavTokenTypes.BOOLEAN,
            ),
            stripped,
        )

        val verbatim = tokens("a: 1\rb: ((\rbody\r))\rkey: true")
        assertEquals(
            listOf(
                "a" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "1" to KtavTokenTypes.INT_VALUE,
                "b" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "((" to KtavTokenTypes.MULTILINE_OPEN,
                "body" to KtavTokenTypes.MULTILINE_TEXT,
                "))" to KtavTokenTypes.MULTILINE_CLOSE,
                "key" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "true" to KtavTokenTypes.BOOLEAN,
            ),
            verbatim,
        )
    }

    @Test
    fun multiline_bodies_close_with_each_line_ending() {
        for (ending in listOf("\n", "\r", "\r\n")) {
            for ((opener, closer) in listOf("(" to ")", "((" to "))")) {
                assertEquals(
                    listOf(
                        "block" to KtavTokenTypes.KEY,
                        ":" to KtavTokenTypes.COLON,
                        opener to KtavTokenTypes.MULTILINE_OPEN,
                        "body" to KtavTokenTypes.MULTILINE_TEXT,
                        closer to KtavTokenTypes.MULTILINE_CLOSE,
                        "next" to KtavTokenTypes.KEY,
                        ":" to KtavTokenTypes.COLON,
                        "false" to KtavTokenTypes.BOOLEAN,
                    ),
                    tokens("block: $opener${ending}body${ending}$closer${ending}next: false"),
                )
            }
        }
    }

    @Test
    fun typed_marker_removed_spec050() {
        // Spec 0.5.0: `:i` is no longer a marker. `port:i 8080` → key `port`,
        // plain `:`, value `i 8080` (a string).
        val toks = tokens("port:i 8080\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("i 8080", toks[2].first)
    }

    @Test
    fun number_value_inferred_from_form() {
        val toks = tokens("port: 8080\n")
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.INT_VALUE, toks[2].second)
        assertEquals("8080", toks[2].first)
    }

    @Test
    fun hex_integer_value() {
        val toks = tokens("mask: 0xFF_00\n")
        assertEquals(KtavTokenTypes.INT_VALUE, toks[2].second)
    }

    @Test
    fun float_value() {
        val toks = tokens("ratio: 1.5e3\n")
        assertEquals(KtavTokenTypes.FLOAT_VALUE, toks[2].second)
    }

    @Test
    fun double_hash_is_comment_single_hash_is_content() {
        val toks = tokens("## a comment\n")
        assertEquals(KtavTokenTypes.COMMENT, toks[0].second)
        assertEquals("## a comment", toks[0].first)
        // Single `#` at line start is NOT a comment (0.5.0).
        val t2 = tokens("#notacomment\n")
        assertEquals(false, t2[0].second == KtavTokenTypes.COMMENT)
    }

    @Test
    fun inline_object_brackets_are_brace_tokens() {
        val toks = tokens("a: {name: alice}\n")
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
        val toks = tokens("xs: [{a: 1}, {b: 2}]\n")
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
        val toks = tokens("a: {win: C:/Users/x}\n")
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
        val toks = tokens("note: hello{world\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("hello{world", toks[2].first)
    }

    @Test
    fun raw_marker_does_not_recognise_keywords() {
        val toks = tokens("flag:: true\n")
        assertEquals(KtavTokenTypes.DOUBLE_COLON, toks[1].second)
        // `true` after `::` must NOT be BOOLEAN
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("true", toks[2].first)
    }

    @Test
    fun emoji_key_is_KEY_token() {
        val toks = tokens("🔧config: enabled\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
    }

    @Test
    fun dotted_key_path() {
        val toks = tokens("a.b.c: value\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)  // a
        assertEquals(KtavTokenTypes.KEY_DOT, toks[1].second)
        assertEquals(KtavTokenTypes.KEY, toks[2].second)  // b
        assertEquals(KtavTokenTypes.KEY_DOT, toks[3].second)
        assertEquals(KtavTokenTypes.KEY, toks[4].second)  // c
    }

    @Test
    fun array_item_without_separator_is_value_not_key() {
        // Inside `xx: [ ... ]` items have no `:` — should be STRING_VALUE.
        val toks = tokens("plainItem\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[0].second)
        assertEquals("plainItem", toks[0].first)
    }

    @Test
    fun array_item_keyword_recognized() {
        val toks = tokens("true\n")
        // `true` on a line with no `:` → keyword, not key
        assertEquals(KtavTokenTypes.BOOLEAN, toks[0].second)
    }

    @Test
    fun ip_address_value_is_string_not_float() {
        // Multiple dots ⇒ not a well-formed number; `127.0.0.1` is a string.
        val toks = tokens("host: 127.0.0.1\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("127.0.0.1", toks[2].first)
    }

    @Test
    fun version_value_is_string_not_float() {
        val toks = tokens("ver: 1.2.3\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("1.2.3", toks[2].first)
    }

    @Test
    fun single_dot_float_still_recognized() {
        val toks = tokens("min_version: 1.3\n")
        assertEquals(KtavTokenTypes.FLOAT_VALUE, toks[2].second)
    }

    @Test
    fun line_starting_with_brace_is_inline_object_array_item() {
        // An inline object as a bare array item must tokenize structurally,
        // not be mis-read as key `{name` + one opaque string value.
        val toks = tokens("{name: alice, age: 30}\n")
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
        val toks = tokens("[1, 2, 3]\n")
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
        val toks = tokens("b: [\n    a, b\n    c, d\n]\n")
        assertEquals(true, toks.any { it.first == "a, b" && it.second == KtavTokenTypes.STRING_VALUE })
        assertEquals(true, toks.any { it.first == "c, d" && it.second == KtavTokenTypes.STRING_VALUE })
        // The comma must NOT be flagged as a bad character.
        assertEquals(false, toks.any { it.second == KtavTokenTypes.BAD_CHARACTER })
    }

    @Test
    fun bare_array_item_with_brackets_is_one_string() {
        // Mid-line `{` / `[` in a bare item are literal content, not structural,
        // so the string highlight isn't split.
        val toks = tokens("arr: [\n    hello{world\n    mid[bracket\n    plain\n]\n")
        assertEquals(true, toks.any { it.first == "hello{world" && it.second == KtavTokenTypes.STRING_VALUE })
        assertEquals(true, toks.any { it.first == "mid[bracket" && it.second == KtavTokenTypes.STRING_VALUE })
        assertEquals(false, toks.any { it.second == KtavTokenTypes.BAD_CHARACTER })
    }

    @Test
    fun bare_dotted_run_is_string_not_dotted_key() {
        // A bare line with dots but no `:` is a string array item (e.g. an IP),
        // not a dotted key.
        val toks = tokens("1.2.3.4\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[0].second)
        assertEquals("1.2.3.4", toks[0].first)
    }

    // ---------------------------------------------------------------
    // Spec 0.6.0: key escaping
    // ---------------------------------------------------------------

    @Test
    fun escaped_dot_in_key_stays_in_KEY_token() {
        // `a\.b: v` — `\.` is a literal dot inside the key, NOT a path
        // separator. Expect ONE KEY token (no KEY_DOT), then COLON, then
        // the value.
        val toks = tokens("a\\.b: v\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("a\\.b", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("v", toks[2].first)
        // No KEY_DOT anywhere.
        assertEquals(false, toks.any { it.second == KtavTokenTypes.KEY_DOT })
    }

    @Test
    fun escaped_colon_in_key_is_not_separator() {
        // `a\:b: v` — the first `:` is escaped; the SECOND `:` is the
        // separator. KEY token must cover `a\:b`.
        val toks = tokens("a\\:b: v\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("a\\:b", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("v", toks[2].first)
    }

    @Test
    fun escaped_backslash_in_key() {
        // `path\\to: v` — `\\` is an escape sequence for a literal `\`.
        // The whole `path\\to` is one KEY token.
        val toks = tokens("path\\\\to: v\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("path\\\\to", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
    }

    @Test
    fun mixed_path_with_escaped_dot_splits_only_unescaped() {
        // `x.y\.z: v` — first `.` is UNESCAPED (KEY_DOT), second `.` is
        // escaped (stays inside KEY `y\.z`).
        val toks = tokens("x.y\\.z: v\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("x", toks[0].first)
        assertEquals(KtavTokenTypes.KEY_DOT, toks[1].second)
        assertEquals(KtavTokenTypes.KEY, toks[2].second)
        assertEquals("y\\.z", toks[2].first)
        assertEquals(KtavTokenTypes.COLON, toks[3].second)
    }

    @Test
    fun escaped_key_in_inline_object() {
        // `obj: {a\.b: 1}` — the inline key `a\.b` is one KEY token
        // (the escaped dot does NOT terminate the key, nor does `\:`).
        val toks = tokens("obj: {a\\.b: 1}\n")
        // obj : { a\.b : 1 }
        assertEquals(KtavTokenTypes.KEY, toks[0].second)         // obj
        assertEquals(KtavTokenTypes.COLON, toks[1].second)       // :
        assertEquals(KtavTokenTypes.LBRACE, toks[2].second)      // {
        assertEquals(KtavTokenTypes.KEY, toks[3].second)         // a\.b
        assertEquals("a\\.b", toks[3].first)
        assertEquals(KtavTokenTypes.COLON, toks[4].second)       // :
        assertEquals(KtavTokenTypes.INT_VALUE, toks[5].second)   // 1
        assertEquals(KtavTokenTypes.RBRACE, toks[6].second)      // }
    }

    // ---------------------------------------------------------------
    // Spec 0.8.0: exact § 3.6 number grammar / § 5.2 redundant-zero rule
    // ---------------------------------------------------------------

    private fun classify(body: String): IElementType {
        val toks = tokens("v: $body\n")
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
        val nbsp = tokens("a:\u00A0true\n")
        assertEquals(KtavTokenTypes.BOOLEAN, nbsp[2].second)
        assertEquals("true", nbsp[2].first)

        val ideographic = tokens("a:\u3000true\n")
        assertEquals(KtavTokenTypes.BOOLEAN, ideographic[2].second)
        assertEquals("true", ideographic[2].first)

        // Trailing whitespace doesn't defeat classification either.
        assertEquals(KtavTokenTypes.BOOLEAN, tokens("a: true\u00A0\n")[2].second)
        assertEquals(KtavTokenTypes.BOOLEAN, tokens("a: true\u3000\n")[2].second)
    }

    // ---------------------------------------------------------------
    // Spec 0.7.0/0.8.0: quoted keys (§ 5.3.3)
    // ---------------------------------------------------------------

    @Test
    fun double_quoted_key_with_embedded_colon() {
        val toks = tokens("\"a:b\": 1\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("\"a:b\"", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.INT_VALUE, toks[2].second)
    }

    @Test
    fun single_quoted_segment_in_dotted_path_with_embedded_dot() {
        val toks = tokens("'x.y'.z: 2\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("'x.y'", toks[0].first)
        assertEquals(KtavTokenTypes.KEY_DOT, toks[1].second)
        assertEquals(KtavTokenTypes.KEY, toks[2].second)
        assertEquals("z", toks[2].first)
        assertEquals(KtavTokenTypes.COLON, toks[3].second)
        assertEquals(KtavTokenTypes.INT_VALUE, toks[4].second)
    }

    @Test
    fun backtick_quoted_key() {
        val toks = tokens("`k`: 3\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("`k`", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.INT_VALUE, toks[2].second)
    }

    @Test
    fun quoted_key_inside_inline_object_with_comma() {
        // The comma inside the quoted key is opaque — one pair, not two.
        val toks = tokens("{\"a,b\": 1}\n")
        val types = toks.map { it.second }
        assertEquals(KtavTokenTypes.LBRACE, types[0])
        assertEquals(KtavTokenTypes.KEY, types[1])
        assertEquals("\"a,b\"", toks[1].first)
        assertEquals(KtavTokenTypes.COLON, types[2])
        assertEquals(KtavTokenTypes.INT_VALUE, types[3])
        assertEquals(KtavTokenTypes.RBRACE, types[4])
        assertEquals(5, toks.size)
    }

    @Test
    fun dotted_bare_scalar_keeps_quoted_colon_opaque_after_spec_whitespace() {
        for (whitespace in listOf(" ", "\u00A0", "\u2028")) {
            for (ending in listOf("\n", "\r", "\r\n")) {
                val scalar = "a.${whitespace}\"b:c\""
                assertEquals(
                    listOf(scalar to KtavTokenTypes.STRING_VALUE),
                    tokens(scalar + ending),
                )
            }
        }
    }

    @Test
    fun dotted_quoted_key_requires_an_external_separator() {
        assertEquals(
            listOf(
                "a. \"b:c\"" to KtavTokenTypes.STRING_VALUE,
                "a" to KtavTokenTypes.KEY,
                "." to KtavTokenTypes.KEY_DOT,
                "\"b:c\"" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "true" to KtavTokenTypes.BOOLEAN,
                "a\\.b" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "1" to KtavTokenTypes.INT_VALUE,
                "bare\"quote" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "2" to KtavTokenTypes.INT_VALUE,
            ),
            tokens("a. \"b:c\"\r\na. \u00A0\"b:c\": true\ra\\.b: 1\nbare\"quote: 2"),
        )
    }

    @Test
    fun quote_inside_bare_inline_key_does_not_swallow_sibling_or_cr() {
        assertEquals(
            listOf(
                "cfg" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "{" to KtavTokenTypes.LBRACE,
                "a\"b" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "1" to KtavTokenTypes.INT_VALUE,
                "," to KtavTokenTypes.COMMA,
                "tail" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "2" to KtavTokenTypes.INT_VALUE,
                "}" to KtavTokenTypes.RBRACE,
                "next" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "true" to KtavTokenTypes.BOOLEAN,
            ),
            tokens("cfg: {a\"b: 1, tail: 2}\rnext: true"),
        )
    }

    @Test
    fun escaped_quote_in_bare_inline_key_and_quoted_segment_after_dot() {
        assertEquals(
            listOf(
                "{" to KtavTokenTypes.LBRACE,
                "a\\\"b" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "1" to KtavTokenTypes.INT_VALUE,
                "," to KtavTokenTypes.COMMA,
                "a.  \"b:c\"" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON,
                "2" to KtavTokenTypes.INT_VALUE,
                "}" to KtavTokenTypes.RBRACE,
            ),
            tokens("{a\\\"b: 1, a.  \"b:c\": 2}"),
        )
    }

    @Test
    fun unterminated_quoted_key_degrades_to_bare_value_no_crash() {
        // No closing `"` before EOL: no separator found ⇒ the whole line
        // is a bare (string) value, not a crash and not a bad-state key.
        val toks = tokens("\"abc: 1\n")
        assertEquals(1, toks.size)
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[0].second)
        assertEquals("\"abc: 1", toks[0].first)
        assertEquals(false, toks.any { it.second == KtavTokenTypes.BAD_CHARACTER })
    }

    // ---------------------------------------------------------------
    // Spec 0.8.0: raw (`::`) values always String; escape forces String
    // ---------------------------------------------------------------

    @Test
    fun raw_value_inside_inline_object_is_always_string() {
        val toks = tokens("{r:: 42}\n")
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
        val toks = tokens("{r:: {open, b: 2}\n")
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
        val toks = tokens("{a: \\u0031}\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[3].second)
    }

    // ---------------------------------------------------------------
    // Incremental relex: restarting at any token boundary with the saved
    // state must reproduce the exact same remaining token stream.
    // ---------------------------------------------------------------

    private fun assertIncrementalRelexStable(text: String) {
        data class Tok(val start: Int, val end: Int, val type: IElementType)

        val toks = mutableListOf<Tok>()
        val stateAfter = mutableListOf<Int>()
        val full = KtavLexer()
        full.start(text, 0, text.length, 0)
        while (full.tokenType != null) {
            toks += Tok(full.tokenStart, full.tokenEnd, full.tokenType!!)
            stateAfter += full.state
            full.advance()
        }

        for (i in toks.indices) {
            val resumeState = if (i == 0) 0 else stateAfter[i - 1]
            val relex = KtavLexer()
            relex.start(text, toks[i].start, text.length, resumeState)
            for (j in i until toks.size) {
                assertEquals("token #$j type when restarting at #$i", toks[j].type, relex.tokenType)
                assertEquals("token #$j start when restarting at #$i", toks[j].start, relex.tokenStart)
                assertEquals("token #$j end when restarting at #$i", toks[j].end, relex.tokenEnd)
                relex.advance()
            }
            assertEquals("no extra trailing token when restarting at #$i", null, relex.tokenType)
        }
    }

    @Test
    fun incremental_relex_stable_across_plain_and_inline_content() {
        assertIncrementalRelexStable(
            "a\\.b: 1\n" +
                "\"c,d\".e: {f:: raw, g: [1, {h: 2}], \"i}j\": 3}\n" +
                "true\n" +
                "## comment\n"
        )
    }

    @Test
    fun incremental_relex_stable_with_unterminated_quoted_key() {
        assertIncrementalRelexStable("\"abc: 1\nnext: 2\n")
    }

    @Test
    fun incremental_relex_stable_across_mixed_line_endings() {
        assertIncrementalRelexStable("a: true\rb: {x: 1}\r\n## note\nc: (\rbody\r)\rnext: false")
    }
}
