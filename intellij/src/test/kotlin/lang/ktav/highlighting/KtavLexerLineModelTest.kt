package lang.ktav.highlighting

import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Pure unit tests for [KtavLexer] — no IntelliJ runtime needed.
 * Line terminators (§ 3.2), BOM (§ 3.1) and multi-line `(`/`((` blocks (§ 5.6).
 */
class KtavLexerLineModelTest {

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
                lexTokens("\"a:b\": true${ending}next: null"),
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
                lexTokens("{v: foo\\${ending}next: true"),
            )
            assertEquals(
                "comment followed by $ending",
                listOf(
                    "## note" to KtavTokenTypes.COMMENT,
                    "next" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON,
                    "true" to KtavTokenTypes.BOOLEAN,
                ),
                lexTokens("## note${ending}next: true"),
            )
        }
    }

    @Test
    fun multiline_bodies_preserve_state_across_cr_and_match_their_closer() {
        val stripped = lexTokens("a: 1\rb: (\rbody\r)\rkey: true")
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

        val verbatim = lexTokens("a: 1\rb: ((\rbody\r))\rkey: true")
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
                    lexTokens("block: $opener${ending}body${ending}$closer${ending}next: false"),
                )
            }
        }
    }

    @Test
    fun parenthesized_array_items_do_not_open_multiline_blocks() {
        assertEquals(
            listOf("(value)" to KtavTokenTypes.STRING_VALUE, "true" to KtavTokenTypes.BOOLEAN),
            lexTokens("(value)\ntrue\n"),
        )
        for (item in listOf("(value)", "((value))", "()", "(())", "(((", "( text", "(( text", "( ## note", "( (")) {
            for (ending in listOf("\n", "\r", "\r\n")) {
                assertArrayItemTokens(
                    "  $item${ending}true${ending}42${ending}1.5${ending}null${ending}",
                    listOf(
                        item to KtavTokenTypes.STRING_VALUE,
                        "true" to KtavTokenTypes.BOOLEAN,
                        "42" to KtavTokenTypes.INT_VALUE,
                        "1.5" to KtavTokenTypes.FLOAT_VALUE,
                        "null" to KtavTokenTypes.NULL,
                    ),
                )
            }
        }
    }

    @Test
    fun exact_array_multiline_openers_accept_only_spec_whitespace() {
        for ((opener, closer) in listOf("(" to ")", "((" to "))")) {
            for (ws in listOf("") + horizontalSpecWhitespace.map { it.toString() }) {
                for (ending in listOf("\n", "\r", "\r\n")) {
                    assertArrayItemTokens(
                        "$ws$opener$ws${ending}true${ending}## body${ending}$ws$closer$ws${ending}false${ending}",
                        listOf(
                            opener to KtavTokenTypes.MULTILINE_OPEN,
                            "true" to KtavTokenTypes.MULTILINE_TEXT,
                            "## body" to KtavTokenTypes.MULTILINE_TEXT,
                            closer to KtavTokenTypes.MULTILINE_CLOSE,
                            "false" to KtavTokenTypes.BOOLEAN,
                        ),
                    )
                }
            }
        }
    }

    @Test
    fun non_spec_whitespace_after_parentheses_is_literal_content() {
        for (ch in listOf('\u0000', '\u001C', '\u001D', '\u001E', '\u001F', '\u180E', '\u200B', '\uFEFF')) {
            for (opener in listOf("(", "((")) {
                val item = opener + ch
                assertArrayItemTokens(
                    "$item\ntrue\n",
                    listOf(item to KtavTokenTypes.STRING_VALUE, "true" to KtavTokenTypes.BOOLEAN),
                )
            }
        }
    }

    @Test
    fun only_the_document_leading_bom_is_metadata() {
        for (ending in listOf("\n", "\r", "\r\n")) {
            for ((opener, closer) in listOf("(" to ")", "((" to "))")) {
                val text = "\uFEFF\u00A0$opener\u3000${ending}true${ending}$closer${ending}false${ending}"
                assertEquals(
                    listOf(
                        opener to KtavTokenTypes.MULTILINE_OPEN,
                        "true" to KtavTokenTypes.MULTILINE_TEXT,
                        closer to KtavTokenTypes.MULTILINE_CLOSE,
                        "false" to KtavTokenTypes.BOOLEAN,
                    ),
                    lexTokens(text),
                )
                assertIncrementalRelexStable(text)
            }
            assertEquals(
                listOf(
                    "(value)" to KtavTokenTypes.STRING_VALUE,
                    "true" to KtavTokenTypes.BOOLEAN,
                    "\uFEFF((" to KtavTokenTypes.STRING_VALUE,
                    "false" to KtavTokenTypes.BOOLEAN,
                ),
                lexTokens("\uFEFF(value)${ending}true${ending}\uFEFF((${ending}false${ending}"),
            )
        }
        assertEquals(
            listOf("\uFEFF(" to KtavTokenTypes.STRING_VALUE),
            lexTokens("\uFEFF\uFEFF("),
        )
        assertEquals(listOf("\uFEFF(" to KtavTokenTypes.STRING_VALUE), lexTokens(" \uFEFF("))
    }

    @Test
    fun raw_array_markers_keep_parentheses_literal() {
        for (item in listOf("(", "((", "(value)", "((value))", "true")) {
            for (ending in listOf("\n", "\r", "\r\n")) {
                assertArrayItemTokens(
                    "::\u00A0$item${ending}true${ending}",
                    listOf(
                        "::" to KtavTokenTypes.DOUBLE_COLON,
                        item to KtavTokenTypes.STRING_VALUE,
                        "true" to KtavTokenTypes.BOOLEAN,
                    ),
                )
                assertEquals(
                    listOf(
                        "raw" to KtavTokenTypes.KEY,
                        "::" to KtavTokenTypes.DOUBLE_COLON,
                        item to KtavTokenTypes.STRING_VALUE,
                        "true" to KtavTokenTypes.BOOLEAN,
                    ),
                    lexTokens("raw:: $item${ending}true${ending}"),
                )
            }
        }
    }

    @Test
    fun pair_values_require_exact_multiline_openers_too() {
        for (item in listOf("(value)", "((value))", "()", "(())", "(((", "(\uFEFF")) {
            assertEquals(
                listOf(
                    "value" to KtavTokenTypes.KEY,
                    ":" to KtavTokenTypes.COLON,
                    item to KtavTokenTypes.STRING_VALUE,
                    "true" to KtavTokenTypes.BOOLEAN,
                ),
                lexTokens("value: $item\ntrue\n"),
            )
        }
    }

    @Test
    fun inline_parentheses_are_strings_even_when_they_are_exact_openers() {
        assertEquals(
            listOf(
                "[" to KtavTokenTypes.LBRACKET,
                "(" to KtavTokenTypes.STRING_VALUE,
                "," to KtavTokenTypes.COMMA,
                "((" to KtavTokenTypes.STRING_VALUE,
                "," to KtavTokenTypes.COMMA,
                "(value)" to KtavTokenTypes.STRING_VALUE,
                "," to KtavTokenTypes.COMMA,
                "((value))" to KtavTokenTypes.STRING_VALUE,
                "]" to KtavTokenTypes.RBRACKET,
                "true" to KtavTokenTypes.BOOLEAN,
            ),
            lexTokens("[(, ((, (value), ((value))]\ntrue\n"),
        )
    }

    @Test
    fun multiline_opener_lookahead_respects_the_buffer_end() {
        val text = "((value))\ntrue\n"
        for ((end, opener) in listOf(1 to "(", 2 to "((")) {
            val lexer = KtavLexer()
            lexer.start(text, 0, end, 0)
            assertEquals(KtavTokenTypes.MULTILINE_OPEN, lexer.tokenType)
            assertEquals(opener, text.substring(lexer.tokenStart, lexer.tokenEnd))
            lexer.advance()
            assertEquals(null, lexer.tokenType)
        }
        for (opener in listOf("(", "((")) {
            assertEquals(listOf(opener to KtavTokenTypes.MULTILINE_OPEN), lexTokens("$opener\u00A0"))
            assertEquals(listOf("$opener\u00A0text" to KtavTokenTypes.STRING_VALUE), lexTokens("$opener\u00A0text"))
        }
    }
}
