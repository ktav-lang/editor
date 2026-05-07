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
        private const val VALUE_STRING = 2  // after `:` — keywords (true/false/null) recognised
        private const val VALUE_INT = 3
        private const val VALUE_FLOAT = 4
        private const val VALUE_RAW = 5     // after `::` — literal text, no keyword recognition

        private val log = com.intellij.openapi.diagnostic.Logger.getInstance(KtavLexer::class.java)
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
        advanceImpl(myBuffer[myTokenStart])
    }

    private fun advanceImpl(c: Char) {

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
            VALUE_STRING -> scanValueString(c, asRaw = false)
            VALUE_RAW -> scanValueString(c, asRaw = true)
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
        // Closing of a multi-line value at line start: `)` / `))`.
        if (c == ')') {
            val isDouble = myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == ')'
            myTokenEnd = if (isDouble) myTokenStart + 2 else myTokenStart + 1
            myTokenType = Tokens.MULTILINE_CLOSE
            return
        }
        // Array item starting with a marker — `::`, `:i`, `:f`, `:` —
        // interpret right here so the value gets its own colour. (Kept
        // separate from `AFTER_KEY` because there's no key on this line.)
        if (c == ':') {
            scanColonMarker()
            return
        }
        // Key character — anything except whitespace and structural chars.
        // Matches the VS Code TextMate regex `[^\s\[\]{}:#.]+`, so non-ASCII
        // letters (Cyrillic, Greek, CJK, emoji) belong to the key the same
        // way as ASCII does. The Ktav spec only forbids the structural set,
        // not the alphabet.
        if (isKeyChar(c)) {
            // Lookahead: if there is no `:` (or `.`) before the next newline,
            // this isn't a key — it's an array item / value line. Tag it as
            // STRING_VALUE so the editor uses string colour (not field/key).
            val isKey = lineHasSeparatorBeforeNewline(myTokenStart)
            scanIdentifier(asKey = isKey)
            myState = if (isKey) AFTER_KEY else LINE_START
            return
        }
        // Anything else → bad char.
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
            isKeyChar(c) -> scanIdentifier(asKey = true)
            else -> {
                myTokenEnd++
                myTokenType = Tokens.BAD_CHARACTER
            }
        }
    }

    /**
     * Char belongs to a key/identifier (everything except whitespace and
     * structural delimiters). Matches the VS Code TextMate behaviour so
     * Cyrillic / CJK / emoji keys highlight the same way.
     */
    private fun isKeyChar(c: Char): Boolean {
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') return false
        return when (c) {
            '{', '}', '[', ']', '(', ')', ':', '#', '.' -> false
            else -> true
        }
    }

    /**
     * Lookahead: does the line starting at [from] contain `:` (or `.`)
     * before the next `\n`? If yes — current ident is a key. If no —
     * it's an array-item / value line, no key on this line.
     *
     * `.` counts because dotted-key paths (`a.b.c:`) still mark the
     * line as a key-pair — `.` is a key continuation.
     */
    private fun lineHasSeparatorBeforeNewline(from: Int): Boolean {
        var i = from
        while (i < myBufferEnd) {
            val ch = myBuffer[i]
            if (ch == '\n') return false
            if (ch == ':' || ch == '.') return true
            i++
        }
        return false
    }

    private fun scanColonMarker() {
        val start = myTokenStart
        val rem = myBufferEnd - start
        // Order matters: ::, :i, :f before plain :
        if (rem >= 2 && myBuffer[start + 1] == ':') {
            myTokenEnd = start + 2
            myTokenType = Tokens.DOUBLE_COLON
            myState = VALUE_RAW  // `::` — literal string, no keyword recognition
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

    private fun scanValueString(c: Char, asRaw: Boolean) {
        if (c == ' ' || c == '\t') {
            scanHorizWhitespace()
            return
        }
        // Compound openers only meaningful for plain `:` values, not `::` raw.
        if (!asRaw) {
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
        }
        // Read value to end of line. Raw form (`::`) skips keyword recognition.
        scanToEndOfLine(recogniseKeywords = !asRaw)
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
            if (isKeyChar(ch)) {
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

    /** Read until end of line. If [recogniseKeywords], `true`/`false`/`null` get keyword tokens. */
    private fun scanToEndOfLine(recogniseKeywords: Boolean) {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd && myBuffer[myTokenEnd] != '\n') {
            myTokenEnd++
        }
        val text = myBuffer.subSequence(myTokenStart, myTokenEnd).toString().trim()
        myTokenType = if (recogniseKeywords) {
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
