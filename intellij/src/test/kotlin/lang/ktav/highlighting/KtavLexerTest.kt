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
        assertEquals(false, toks.any { it.second == TokenType.BAD_CHARACTER })
    }

    @Test
    fun bare_array_item_with_brackets_is_one_string() {
        // Mid-line `{` / `[` in a bare item are literal content, not structural,
        // so the string highlight isn't split.
        val toks = tokens("arr: [\n    hello{world\n    mid[bracket\n    plain\n]\n")
        assertEquals(true, toks.any { it.first == "hello{world" && it.second == KtavTokenTypes.STRING_VALUE })
        assertEquals(true, toks.any { it.first == "mid[bracket" && it.second == KtavTokenTypes.STRING_VALUE })
        assertEquals(false, toks.any { it.second == TokenType.BAD_CHARACTER })
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
}
