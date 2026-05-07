package lang.ktav.highlighting

import com.intellij.lexer.LexerBase
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import lang.ktav.highlighting.KtavTokenTypes as Tokens

/**
 * State-machine lexer for Ktav syntax highlighting.
 *
 * Ktav is line-oriented (`key:i 5`, `key: text`, `key:: raw`, `key.sub: {`).
 * The lexer tracks per-line state so that, e.g., `10` after `:i` is
 * tokenised as INT_VALUE while `10` after `:` (no marker) is STRING_VALUE.
 *
 * States encode "what comes next on the current logical line":
 *   - LINE_START   : at column 0 (or after newline) — expect KEY / `{` / `}` / `[` / `]`
 *   - AFTER_KEY    : key segment seen — expect `.subkey`, more KEY chars, or `:` marker
 *   - VALUE_STRING : after `:` (no marker) or `::` — rest of line is STRING_VALUE
 *   - VALUE_INT    : after `:i` — rest of line is INT_VALUE
 *   - VALUE_FLOAT  : after `:f` — rest of line is FLOAT_VALUE
 *
 * State is preserved in `myState` so PsiBuilder's incremental relexing works.
 */
class KtavLexer : LexerBase() {

    companion object {
        private const val LINE_START = 0
        private const val AFTER_KEY = 1
        private const val VALUE_STRING = 2
        private const val VALUE_INT = 3
        private const val VALUE_FLOAT = 4
    }

    private var myBuffer: CharSequence = ""
    private var myBufferEnd = 0
    private var myState = LINE_START
    private var myTokenStart = 0
    private var myTokenEnd = 0
    private var myTokenType: IElementType? = null

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
        myBuffer = buffer
        myBufferEnd = endOffset
        myState = initialState
        myTokenStart = startOffset
        myTokenEnd = startOffset
        advance()
    }

    override fun getState() = myState
    override fun getTokenType(): IElementType? = myTokenType
    override fun getTokenStart() = myTokenStart
    override fun getTokenEnd() = myTokenEnd
    override fun getBufferSequence() = myBuffer
    override fun getBufferEnd() = myBufferEnd

    override fun advance() {
        myTokenStart = myTokenEnd
        if (myTokenStart >= myBufferEnd) {
            myTokenType = null
            return
        }

        val c = myBuffer[myTokenStart]

        // Newlines reset state to LINE_START regardless of previous state.
        if (c == '\n') {
            myTokenEnd = myTokenStart + 1
            myTokenType = TokenType.WHITE_SPACE
            myState = LINE_START
            return
        }

        // Comment lines start with `#` — they bind tighter than state.
        if (c == '#' && myState == LINE_START) {
            scanCommentRest()
            return
        }

        when (myState) {
            LINE_START -> scanLineStart(c)
            AFTER_KEY -> scanAfterKey(c)
            VALUE_STRING -> scanValueString(c)
            VALUE_INT -> scanValueInt(c)
            VALUE_FLOAT -> scanValueFloat(c)
            else -> scanLineStart(c)
        }
    }

    // -------------------------------------------------------------------
    // State handlers
    // -------------------------------------------------------------------

    private fun scanLineStart(c: Char) {
        // Whitespace at line start
        if (c == ' ' || c == '\t') {
            scanHorizWhitespace()
            return
        }
        // Structural characters at line start (compound open/close lines)
        when (c) {
            '{' -> { myTokenEnd++; myTokenType = Tokens.LBRACE; return }
            '}' -> { myTokenEnd++; myTokenType = Tokens.RBRACE; return }
            '[' -> { myTokenEnd++; myTokenType = Tokens.LBRACKET; return }
            ']' -> { myTokenEnd++; myTokenType = Tokens.RBRACKET; return }
        }
        // Identifier (key) — ASCII letters/digits/underscore.
        if (c.isLetter() || c == '_' || c.isDigit() || c == '-') {
            scanIdentifier(asKey = true)
            myState = AFTER_KEY
            return
        }
        // Anything else at line start → bad char (unexpected)
        myTokenEnd++
        myTokenType = Tokens.BAD_CHARACTER
    }

    private fun scanAfterKey(c: Char) {
        when {
            c == '.' -> {
                myTokenEnd++
                myTokenType = Tokens.KEY_DOT
                // Stay in AFTER_KEY; next ident segment continues the key path.
            }
            c == ':' -> scanColonMarker()
            c == ' ' || c == '\t' -> scanHorizWhitespace()
            c.isLetter() || c == '_' || c.isDigit() || c == '-' -> {
                scanIdentifier(asKey = true)
            }
            else -> {
                myTokenEnd++
                myTokenType = Tokens.BAD_CHARACTER
            }
        }
    }

    private fun scanColonMarker() {
        val start = myTokenStart
        val rem = myBufferEnd - start
        // Order matters: ::, :i, :f before plain :
        if (rem >= 2 && myBuffer[start + 1] == ':') {
            myTokenEnd = start + 2
            myTokenType = Tokens.DOUBLE_COLON
            myState = VALUE_STRING
        } else if (rem >= 2 && myBuffer[start + 1] == 'i'
            && (rem == 2 || isMarkerBoundary(myBuffer[start + 2]))) {
            myTokenEnd = start + 2
            myTokenType = Tokens.MARKER_INT
            myState = VALUE_INT
        } else if (rem >= 2 && myBuffer[start + 1] == 'f'
            && (rem == 2 || isMarkerBoundary(myBuffer[start + 2]))) {
            myTokenEnd = start + 2
            myTokenType = Tokens.MARKER_FLOAT
            myState = VALUE_FLOAT
        } else {
            myTokenEnd = start + 1
            myTokenType = Tokens.COLON
            myState = VALUE_STRING
        }
    }

    private fun scanValueString(c: Char) {
        if (c == ' ' || c == '\t') {
            scanHorizWhitespace()
            return
        }
        // Compound openers on value line — let parent state machine handle
        // `{` `[` `(` (multi-line). Here we just pass them as structural.
        when (c) {
            '{' -> { myTokenEnd++; myTokenType = Tokens.LBRACE; return }
            '[' -> { myTokenEnd++; myTokenType = Tokens.LBRACKET; return }
            '(' -> {
                // Multi-line open: `(` or `((`
                val isDouble = myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == '('
                myTokenEnd = if (isDouble) myTokenStart + 2 else myTokenStart + 1
                myTokenType = Tokens.MULTILINE_OPEN
                return
            }
        }
        // Read string value to end of line (trim newlines via state reset).
        scanToEndOfLine(asValue = true)
    }

    private fun scanValueInt(c: Char) {
        if (c == ' ' || c == '\t') {
            scanHorizWhitespace()
            return
        }
        // Read integer literal: optional sign + digits (followed by EOL).
        scanNumericValue(Tokens.INT_VALUE)
    }

    private fun scanValueFloat(c: Char) {
        if (c == ' ' || c == '\t') {
            scanHorizWhitespace()
            return
        }
        scanNumericValue(Tokens.FLOAT_VALUE)
    }

    // -------------------------------------------------------------------
    // Token-level scanners
    // -------------------------------------------------------------------

    private fun scanIdentifier(asKey: Boolean) {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd) {
            val ch = myBuffer[myTokenEnd]
            if (ch.isLetterOrDigit() || ch == '_' || ch == '-') {
                myTokenEnd++
            } else break
        }
        myTokenType = if (asKey) Tokens.KEY else {
            val text = myBuffer.subSequence(myTokenStart, myTokenEnd).toString()
            when (text) {
                "true", "false" -> Tokens.BOOLEAN
                "null" -> Tokens.NULL
                else -> Tokens.STRING_VALUE
            }
        }
    }

    private fun scanCommentRest() {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd && myBuffer[myTokenEnd] != '\n') {
            myTokenEnd++
        }
        myTokenType = Tokens.COMMENT
    }

    /** Scan whitespace that doesn't include newlines (those reset state separately). */
    private fun scanHorizWhitespace() {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd) {
            val ch = myBuffer[myTokenEnd]
            if (ch == ' ' || ch == '\t') myTokenEnd++ else break
        }
        myTokenType = TokenType.WHITE_SPACE
    }

    /** Read until end of line, classify as appropriate value token. */
    private fun scanToEndOfLine(asValue: Boolean) {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd && myBuffer[myTokenEnd] != '\n') {
            myTokenEnd++
        }
        val text = myBuffer.subSequence(myTokenStart, myTokenEnd).toString().trim()
        myTokenType = if (asValue) {
            when (text) {
                "true", "false" -> Tokens.BOOLEAN
                "null" -> Tokens.NULL
                else -> Tokens.STRING_VALUE
            }
        } else Tokens.STRING_VALUE
    }

    private fun scanNumericValue(tokenType: IElementType) {
        myTokenEnd = myTokenStart
        // Whole rest of line — let LSP/parser flag malformed numbers; we just colour.
        while (myTokenEnd < myBufferEnd && myBuffer[myTokenEnd] != '\n') {
            myTokenEnd++
        }
        myTokenType = tokenType
    }

    /** A char that legitimately follows `:i` / `:f` (whitespace or EOL). */
    private fun isMarkerBoundary(c: Char): Boolean = c == ' ' || c == '\t' || c == '\n'
}
