package lang.ktav.highlighting

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Pure unit tests for [KtavLexer] — no IntelliJ runtime needed.
 * Incremental relex stability and block/scope context across restarts.
 */
class KtavLexerIncrementalTest {

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
    @Test
    fun array_pair_shaped_strings_do_not_open_blocks_and_object_context_returns_after_close() {
        for (ending in listOf("\n", "\r", "\r\n")) {
            val text = listOf("items: [", "name: (", "true", "[", "name: true", "]", "]", "after: 1")
                .joinToString(ending, postfix = ending)
            assertEquals(
                listOf(
                    "items" to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON,
                    "[" to KtavTokenTypes.LBRACKET,
                    "name: (" to KtavTokenTypes.STRING_VALUE,
                    "true" to KtavTokenTypes.BOOLEAN,
                    "[" to KtavTokenTypes.LBRACKET,
                    "name: true" to KtavTokenTypes.STRING_VALUE,
                    "]" to KtavTokenTypes.RBRACKET,
                    "]" to KtavTokenTypes.RBRACKET,
                    "after" to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON,
                    "1" to KtavTokenTypes.INT_VALUE,
                ),
                lexTokens(text),
            )
            assertIncrementalRelexStable(text)
        }
    }

    @Test
    fun first_content_keeps_implicit_array_context_across_nested_objects_and_multiline_strings() {
        val text = "\uFEFF\u00A0## comment\r\nhello\rname: (\n{\rx: [1, true]\n}\r\n((\r{\n))\rname: true\n"
        assertEquals(
            listOf(
                "## comment" to KtavTokenTypes.COMMENT,
                "hello" to KtavTokenTypes.STRING_VALUE,
                "name: (" to KtavTokenTypes.STRING_VALUE,
                "{" to KtavTokenTypes.LBRACE,
                "x" to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON,
                "[" to KtavTokenTypes.LBRACKET, "1" to KtavTokenTypes.INT_VALUE,
                "," to KtavTokenTypes.COMMA, "true" to KtavTokenTypes.BOOLEAN,
                "]" to KtavTokenTypes.RBRACKET, "}" to KtavTokenTypes.RBRACE,
                "((" to KtavTokenTypes.MULTILINE_OPEN, "{" to KtavTokenTypes.MULTILINE_TEXT,
                "))" to KtavTokenTypes.MULTILINE_CLOSE,
                "name: true" to KtavTokenTypes.STRING_VALUE,
            ),
            lexTokens(text),
        )
        assertIncrementalRelexStable(text)
    }

    @Test
    fun explicit_roots_and_glued_first_scalar_keep_their_kind() {
        for (ending in listOf("\n", "\r", "\r\n")) {
            for (first in listOf("hello", "key:value", "a. \"b:c\"", "\"unterminated")) {
                val text = "$first${ending}name: true${ending}"
                assertEquals(
                    listOf(first to KtavTokenTypes.STRING_VALUE, "name: true" to KtavTokenTypes.STRING_VALUE),
                    lexTokens(text),
                )
                assertIncrementalRelexStable(text)
            }
            val array = "[${ending}name: true${ending}]${ending}"
            assertEquals(
                listOf("[" to KtavTokenTypes.LBRACKET, "name: true" to KtavTokenTypes.STRING_VALUE,
                    "]" to KtavTokenTypes.RBRACKET),
                lexTokens(array),
            )
            val obj = "{${ending}name: true${ending}}${ending}"
            assertEquals(
                listOf("{" to KtavTokenTypes.LBRACE, "name" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON, "true" to KtavTokenTypes.BOOLEAN,
                    "}" to KtavTokenTypes.RBRACE),
                lexTokens(obj),
            )
            assertIncrementalRelexStable(array)
            assertIncrementalRelexStable(obj)
        }
    }

    @Test
    fun positional_quotes_inside_multiword_bare_key_segments_preserve_separator_and_value() {
        for (quote in listOf('"', '\'', '`')) {
            for (ws in horizontalSpecWhitespace) {
                val key = "first${ws}${quote}name"
                val text = "$key: 1\nouter . \\\\part${ws}${quote}tail . \"x:y\": false\r\nnext: true\n"
                assertEquals(
                    listOf(
                        key to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON, "1" to KtavTokenTypes.INT_VALUE,
                        "outer" to KtavTokenTypes.KEY, "." to KtavTokenTypes.KEY_DOT,
                        "\\\\part${ws}${quote}tail" to KtavTokenTypes.KEY, "." to KtavTokenTypes.KEY_DOT,
                        "\"x:y\"" to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON,
                        "false" to KtavTokenTypes.BOOLEAN,
                        "next" to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON,
                        "true" to KtavTokenTypes.BOOLEAN,
                    ),
                    lexTokens(text),
                )
                assertIncrementalRelexStable(text)
                val inline = "{$key: 1, next: true}"
                assertEquals(
                    listOf(
                        "{" to KtavTokenTypes.LBRACE, key to KtavTokenTypes.KEY,
                        ":" to KtavTokenTypes.COLON, "1" to KtavTokenTypes.INT_VALUE,
                        "," to KtavTokenTypes.COMMA, "next" to KtavTokenTypes.KEY,
                        ":" to KtavTokenTypes.COLON, "true" to KtavTokenTypes.BOOLEAN,
                        "}" to KtavTokenTypes.RBRACE,
                    ),
                    lexTokens(inline),
                )
                assertIncrementalRelexStable(inline)
            }
        }
    }

    @Test
    fun incremental_relex_preserves_deep_multiline_and_inline_container_stacks() {
        val text = "items: [\n" + "[\n".repeat(40) +
            "{first \"name: [1, {flag: true}]}\n" +
            "]\n".repeat(41) + "after: 2\n"
        assertIncrementalRelexStable(text)
        assertIncrementalRelexStable("value: " + "[".repeat(40) + "{flag: true}" + "]".repeat(40) + "\nnext: 2")
    }

    @Test
    fun incremental_edits_do_not_converge_between_object_and_array_contexts() {
        val lexer = KtavLexer()
        val before = "items: {\nname: true\n}\nafter: 1\n"
        lexer.start(before, 0, before.length, 0)
        val objectStates = mutableListOf<Int>()
        while (lexer.tokenType != null) {
            if (before.substring(lexer.tokenStart, lexer.tokenEnd) == "name") objectStates += lexer.state
            lexer.advance()
        }
        val after = "items: [\nname: true\n]\nafter: 1\n"
        lexer.start(after, 0, after.length, 0)
        while (lexer.tokenType != null) {
            if (after.substring(lexer.tokenStart, lexer.tokenEnd) == "name: true") {
                org.junit.Assert.assertNotEquals(objectStates.single(), lexer.state)
            }
            lexer.advance()
        }
        assertEquals(
            listOf("items" to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON,
                "[" to KtavTokenTypes.LBRACKET, "name: true" to KtavTokenTypes.STRING_VALUE,
                "]" to KtavTokenTypes.RBRACKET, "after" to KtavTokenTypes.KEY,
                ":" to KtavTokenTypes.COLON, "1" to KtavTokenTypes.INT_VALUE),
            lexTokens(after),
        )
        assertIncrementalRelexStable(after)
    }

    @Test
    fun lone_paren_closers_are_whole_array_strings_not_block_delimiters() {
        val values = listOf(
            ")" to KtavTokenTypes.STRING_VALUE,
            "))" to KtavTokenTypes.STRING_VALUE,
            ") suffix" to KtavTokenTypes.STRING_VALUE,
        )
        val bare = ")\n))\n) suffix\n"
        val nested = "items: [\n$bare]\nafter: 1\n"
        for ((text, expected) in listOf(
            bare to values,
            nested to (listOf("items" to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON,
                "[" to KtavTokenTypes.LBRACKET) + values +
                listOf("]" to KtavTokenTypes.RBRACKET, "after" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON, "1" to KtavTokenTypes.INT_VALUE)),
        )) {
            assertEquals(expected, lexTokens(text))
            assertIncrementalRelexStable(text)
        }
    }
}
