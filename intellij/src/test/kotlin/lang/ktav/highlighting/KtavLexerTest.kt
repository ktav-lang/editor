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
    fun typed_int_marker_recognised() {
        val toks = tokens("port:i 8080\n")
        assertEquals(KtavTokenTypes.KEY, toks[0].second)
        assertEquals(KtavTokenTypes.MARKER_INT, toks[1].second)
        assertEquals(KtavTokenTypes.INT_VALUE, toks[2].second)
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
}
