package lang.ktav.highlighting

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Pure unit tests for [KtavLexer] — no IntelliJ runtime needed.
 * Keys: bare, dotted, escaped (§ 3.7) and quoted (§ 5.3.3) segments.
 */
class KtavLexerKeysTest {

    @Test
    fun cyrillic_key_is_KEY_token() {
        val toks = lexTokens("имя: Иван\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("имя", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[2].second)
        assertEquals("Иван", toks[2].first)
    }

    @Test
    fun ascii_key_is_KEY_token() {
        val toks = lexTokens("name: John\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("name", toks[0].first)
    }

    @Test
    fun emoji_key_is_KEY_token() {
        val toks = lexTokens("🔧config: enabled\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
    }

    @Test
    fun dotted_key_path() {
        val toks = lexTokens("a.b.c: value\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)  // a
        assertEquals(KtavTokenTypes.KEY_DOT, toks[1].second)
        assertEquals(KtavTokenTypes.KEY, toks[2].second)  // b
        assertEquals(KtavTokenTypes.KEY_DOT, toks[3].second)
        assertEquals(KtavTokenTypes.KEY, toks[4].second)  // c
    }

    @Test
    fun array_item_without_separator_is_value_not_key() {
        // Inside `xx: [ ... ]` items have no `:` — should be STRING_VALUE.
        val toks = lexTokens("plainItem\n")
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[0].second)
        assertEquals("plainItem", toks[0].first)
    }

    // ---------------------------------------------------------------
    // Spec 0.6.0: key escaping
    // ---------------------------------------------------------------

    @Test
    fun escaped_dot_in_key_stays_in_KEY_token() {
        // `a\.b: v` — `\.` is a literal dot inside the key, NOT a path
        // separator. Expect ONE KEY token (no KEY_DOT), then COLON, then
        // the value.
        val toks = lexTokens("a\\.b: v\n")
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
        val toks = lexTokens("a\\:b: v\n")
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
        val toks = lexTokens("path\\\\to: v\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("path\\\\to", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
    }

    @Test
    fun mixed_path_with_escaped_dot_splits_only_unescaped() {
        // `x.y\.z: v` — first `.` is UNESCAPED (KEY_DOT), second `.` is
        // escaped (stays inside KEY `y\.z`).
        val toks = lexTokens("x.y\\.z: v\n")
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
        val toks = lexTokens("obj: {a\\.b: 1}\n")
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
    // Spec 0.7.0/0.8.0: quoted keys (§ 5.3.3)
    // ---------------------------------------------------------------

    @Test
    fun double_quoted_key_with_embedded_colon() {
        val toks = lexTokens("\"a:b\": 1\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("\"a:b\"", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.INT_VALUE, toks[2].second)
    }

    @Test
    fun single_quoted_segment_in_dotted_path_with_embedded_dot() {
        val toks = lexTokens("'x.y'.z: 2\n")
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
        val toks = lexTokens("`k`: 3\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals("`k`", toks[0].first)
        assertEquals(KtavTokenTypes.COLON, toks[1].second)
        assertEquals(KtavTokenTypes.INT_VALUE, toks[2].second)
    }

    @Test
    fun quoted_key_inside_inline_object_with_comma() {
        // The comma inside the quoted key is opaque — one pair, not two.
        val toks = lexTokens("{\"a,b\": 1}\n")
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
                    lexTokens(scalar + ending),
                )
            }
        }
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
            lexTokens("cfg: {a\"b: 1, tail: 2}\rnext: true"),
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
            lexTokens("{a\\\"b: 1, a.  \"b:c\": 2}"),
        )
    }

    @Test
    fun unterminated_quoted_key_degrades_to_bare_value_no_crash() {
        // No closing `"` before EOL: no separator found ⇒ the whole line
        // is a bare (string) value, not a crash and not a bad-state key.
        val toks = lexTokens("\"abc: 1\n")
        assertEquals(1, toks.size)
        assertEquals(KtavTokenTypes.STRING_VALUE, toks[0].second)
        assertEquals("\"abc: 1", toks[0].first)
        assertEquals(false, toks.any { it.second == KtavTokenTypes.BAD_CHARACTER })
    }
}
