package lang.ktav.highlighting

import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import org.junit.Assert.assertEquals

/** Shared helpers for the [KtavLexer] test classes. */


/** Run lexer over [text]; return list of (text, token) pairs (whitespace skipped). */
internal fun lexTokens(text: String): List<Pair<String, IElementType>> {
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

// ---------------------------------------------------------------
// Incremental relex: restarting at any token boundary with the saved
// state must reproduce the exact same remaining token stream.
// ---------------------------------------------------------------

internal fun assertIncrementalRelexStable(text: String) {
    data class Tok(val start: Int, val end: Int, val type: IElementType)

    val toks = mutableListOf<Tok>()
    val states = mutableListOf<Int>()
    val full = KtavLexer()
    full.start(text, 0, text.length, 0)
    while (full.tokenType != null) {
        toks += Tok(full.tokenStart, full.tokenEnd, full.tokenType!!)
        states += full.state
        full.advance()
    }

    for (i in toks.indices) {
        for (relex in listOf(KtavLexer(), full)) {
            relex.start(text, toks[i].start, text.length, states[i])
            for (j in i until toks.size) {
                assertEquals("token #$j type when restarting at #$i", toks[j].type, relex.tokenType)
                assertEquals("token #$j start when restarting at #$i", toks[j].start, relex.tokenStart)
                assertEquals("token #$j end when restarting at #$i", toks[j].end, relex.tokenEnd)
                relex.advance()
            }
            assertEquals("no extra trailing token when restarting at #$i", null, relex.tokenType)
        }
    }
}



internal fun assertArrayItemTokens(body: String, expected: List<Pair<String, IElementType>>) {
    assertEquals(expected, lexTokens(body))
    assertIncrementalRelexStable(body)
    for ((prefix, prefixTokens) in listOf(
        "[\n" to listOf("[" to KtavTokenTypes.LBRACKET),
        "items: [\n" to listOf(
            "items" to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON,
            "[" to KtavTokenTypes.LBRACKET,
        ),
        "items: [\n[\n" to listOf(
            "items" to KtavTokenTypes.KEY, ":" to KtavTokenTypes.COLON,
            "[" to KtavTokenTypes.LBRACKET, "[" to KtavTokenTypes.LBRACKET,
        ),
    )) {
        val closers = if (prefix == "items: [\n[\n") 2 else 1
        val text = prefix + body + "]\n".repeat(closers)
        assertEquals(
            prefixTokens + expected + List(closers) { "]" to KtavTokenTypes.RBRACKET },
            lexTokens(text),
        )
        assertIncrementalRelexStable(text)
    }
}

internal val horizontalSpecWhitespace = listOf(
    '\u0009', '\u000B', '\u000C', '\u0020', '\u0085', '\u00A0', '\u1680',
    '\u2028', '\u2029', '\u202F', '\u205F', '\u3000',
) + ('\u2000'..'\u200A')
