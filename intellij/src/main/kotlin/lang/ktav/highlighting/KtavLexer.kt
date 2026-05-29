package lang.ktav.highlighting

import com.intellij.lexer.LexerBase
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import lang.ktav.highlighting.KtavTokenTypes as Tokens

/**
 * State-machine lexer for Ktav syntax highlighting (spec 0.5.0).
 *
 * Ktav is line-oriented (`key: text`, `key:: raw`, `key.sub: {`, `## comment`).
 * Spec 0.5.0 changes baked in here:
 *   - Comments require `##` (two hashes); a single `#` is ordinary content
 *     (allowed in keys and values).
 *   - Typed markers `:i` / `:f` are gone. Only `:` (string/value) and `::`
 *     (raw literal string) remain. Numbers are inferred from the scalar's
 *     surface form (hex / oct / bin / decimal / float), mirroring the LSP.
 *   - Inline compounds (`{k: v, ...}`, `[v1, v2]`, nesting allowed) are
 *     tokenised structurally so their brackets become LBRACE/LBRACKET/…
 *     tokens — that is what powers brace-matching for inline objects.
 *
 * Line states: LINE_START, AFTER_KEY, VALUE_STRING, VALUE_RAW.
 *
 * Inline state: when a `:` value begins with `{` or `[`, the lexer descends
 * into an inline-tokenising mode. The whole nesting context (depth, the
 * object/array stack, and whether a key or a value is expected next) is
 * packed into the integer lexer state so IntelliJ's incremental relexing
 * stays correct when it restarts at any inline token boundary.
 */
class KtavLexer : LexerBase() {

    companion object {
        private const val LINE_START = 0
        private const val AFTER_KEY = 1
        private const val VALUE_STRING = 2  // after `:` — keyword/number recognition
        private const val VALUE_RAW = 3     // after `::` — literal text, no recognition

        // Inline states occupy everything >= INLINE_BASE. The remainder
        // encodes: bit0 = expectKey, bits1..5 = depth, bits6+ = container
        // stack (LSB = innermost; 1 = object, 0 = array).
        private const val INLINE_BASE = 16
        private const val MAX_INLINE_DEPTH = 24

        private fun encodeInline(depth: Int, stack: Int, expectKey: Boolean): Int {
            val d = depth.coerceIn(0, MAX_INLINE_DEPTH)
            return INLINE_BASE + (if (expectKey) 1 else 0) + (d shl 1) + (stack shl 6)
        }

        private fun isInline(state: Int) = state >= INLINE_BASE
        private fun inDepth(state: Int) = ((state - INLINE_BASE) shr 1) and 0x1F
        private fun inStack(state: Int) = (state - INLINE_BASE) shr 6
        private fun inExpectKey(state: Int) = ((state - INLINE_BASE) and 1) == 1
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
        // Newlines reset state to LINE_START regardless of previous state
        // (inline compounds are single-line; an unterminated one resets).
        if (c == '\n') {
            myTokenEnd = myTokenStart + 1
            myTokenType = TokenType.WHITE_SPACE
            myState = LINE_START
            return
        }

        // Comments: `##` at line start. A single `#` is ordinary content.
        if (c == '#' && myState == LINE_START
            && myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == '#') {
            scanCommentRest()
            return
        }

        if (isInline(myState)) {
            scanInline(c)
            return
        }

        when (myState) {
            LINE_START -> scanLineStart(c)
            AFTER_KEY -> scanAfterKey(c)
            VALUE_STRING -> scanValueString(c, asRaw = false)
            VALUE_RAW -> scanValueString(c, asRaw = true)
            else -> scanLineStart(c)
        }
    }

    // -------------------------------------------------------------------
    // Line-level state handlers
    // -------------------------------------------------------------------

    private fun scanLineStart(c: Char) {
        if (c == ' ' || c == '\t') {
            scanHorizWhitespace()
            return
        }
        when (c) {
            // A line beginning with `{` / `[` is an inline-compound value
            // (an array item like `{name: alice, age: 30}`), not a key. Enter
            // the inline tokenizer so the whole compound is broken into
            // structural + scalar sub-tokens (mirrors `scanValueString`).
            '{' -> { myTokenEnd++; myTokenType = Tokens.LBRACE
                myState = encodeInline(1, 1, expectKey = true); return }
            '[' -> { myTokenEnd++; myTokenType = Tokens.LBRACKET
                myState = encodeInline(1, 0, expectKey = false); return }
            // Lone closers stay plain braces and keep the line-start state.
            '}' -> { myTokenEnd++; myTokenType = Tokens.RBRACE; return }
            ']' -> { myTokenEnd++; myTokenType = Tokens.RBRACKET; return }
        }
        if (c == ')') {
            val isDouble = myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == ')'
            myTokenEnd = if (isDouble) myTokenStart + 2 else myTokenStart + 1
            myTokenType = Tokens.MULTILINE_CLOSE
            return
        }
        // Array item / pair starting with a marker (`::` or `:`).
        if (c == ':') {
            scanColonMarker()
            return
        }
        if (isKeyChar(c)) {
            if (lineHasSeparatorBeforeNewline(myTokenStart)) {
                scanIdentifier(asKey = true)
                myState = AFTER_KEY
            } else {
                // No `:` on this line ⇒ a bare array-item scalar. The WHOLE
                // trimmed line is one value; commas / brackets / dots here are
                // literal string content (commas separate items only inside an
                // INLINE `[a, b]`). Reading to EOL keeps `a, b` and `1.2.3.4`
                // one token instead of flagging the comma as a bad character
                // or splitting on dots. Mirrors the LSP's per-line classifier.
                scanToEndOfLine(recognise = true)
                myState = LINE_START
            }
            return
        }
        myTokenEnd++
        myTokenType = Tokens.BAD_CHARACTER
    }

    private fun scanAfterKey(c: Char) {
        when {
            c == '.' -> {
                myTokenEnd++
                myTokenType = Tokens.KEY_DOT
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
     * Char belongs to a key/identifier — everything except whitespace and
     * the structural delimiters. `#` IS a key char in 0.5.0 (a lone `#` is
     * content); only `##` at line start opens a comment. Non-ASCII letters
     * (Cyrillic / CJK / emoji) belong to keys just like ASCII.
     */
    private fun isKeyChar(c: Char): Boolean {
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') return false
        return when (c) {
            '{', '}', '[', ']', '(', ')', ':', '.', ',' -> false
            else -> true
        }
    }

    private fun lineHasSeparatorBeforeNewline(from: Int): Boolean {
        var i = from
        while (i < myBufferEnd) {
            val ch = myBuffer[i]
            if (ch == '\n') return false
            // Only `:` makes a line a `key: value` pair. A `.` alone does not
            // (a bare dotted run like `1.2.3.4` is a string array item, not a
            // dotted key — the dotted key still needs its `:`).
            if (ch == ':') return true
            i++
        }
        return false
    }

    private fun scanColonMarker() {
        val start = myTokenStart
        val rem = myBufferEnd - start
        if (rem >= 2 && myBuffer[start + 1] == ':') {
            myTokenEnd = start + 2
            myTokenType = Tokens.DOUBLE_COLON
            myState = VALUE_RAW
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
        if (!asRaw) {
            when (c) {
                '{' -> { myTokenEnd++; myTokenType = Tokens.LBRACE
                    myState = encodeInline(1, 1, expectKey = true); return }
                '[' -> { myTokenEnd++; myTokenType = Tokens.LBRACKET
                    myState = encodeInline(1, 0, expectKey = false); return }
                '(' -> {
                    val isDouble = myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == '('
                    myTokenEnd = if (isDouble) myTokenStart + 2 else myTokenStart + 1
                    myTokenType = Tokens.MULTILINE_OPEN
                    return
                }
            }
        }
        // Plain scalar value: whole rest of line. Raw (`::`) skips recognition.
        scanToEndOfLine(recognise = !asRaw)
    }

    // -------------------------------------------------------------------
    // Inline-compound tokeniser
    // -------------------------------------------------------------------

    private fun scanInline(c: Char) {
        val depth = inDepth(myState)
        val stack = inStack(myState)
        val expectKey = inExpectKey(myState)

        if (c == ' ' || c == '\t') {
            scanHorizWhitespace()
            return
        }
        when (c) {
            '{' -> {
                myTokenEnd = myTokenStart + 1
                myTokenType = Tokens.LBRACE
                myState = encodeInline(depth + 1, (stack shl 1) or 1, expectKey = true)
                return
            }
            '[' -> {
                myTokenEnd = myTokenStart + 1
                myTokenType = Tokens.LBRACKET
                myState = encodeInline(depth + 1, stack shl 1, expectKey = false)
                return
            }
            '}' -> { closeInline(Tokens.RBRACE, depth, stack); return }
            ']' -> { closeInline(Tokens.RBRACKET, depth, stack); return }
            ',' -> {
                myTokenEnd = myTokenStart + 1
                myTokenType = Tokens.COMMA
                val containerIsObject = (stack and 1) == 1
                myState = encodeInline(depth, stack, expectKey = containerIsObject)
                return
            }
            ':' -> {
                // Separator inside an object pair (only meaningful when a key
                // was just read). `::` raw marker also possible.
                val dbl = myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == ':'
                myTokenEnd = if (dbl) myTokenStart + 2 else myTokenStart + 1
                myTokenType = if (dbl) Tokens.DOUBLE_COLON else Tokens.COLON
                myState = encodeInline(depth, stack, expectKey = false)
                return
            }
        }
        if (expectKey) {
            // Key run: up to a structural delimiter.
            var e = myTokenStart
            while (e < myBufferEnd) {
                val ch = myBuffer[e]
                if (ch == '\n' || ch == ':' || ch == ',' || ch == '{' || ch == '}' || ch == '[' || ch == ']') break
                e++
            }
            myTokenEnd = e
            myTokenType = Tokens.KEY
            // expectKey stays true until the `:` separator flips it.
        } else {
            // Value scalar: up to an unescaped `,` / `}` / `]`. `{` / `[`
            // mid-run are literal content (head/rest rule); a value that
            // *begins* with `{` / `[` was already handled as a nested
            // compound by the dispatch above.
            var e = myTokenStart
            while (e < myBufferEnd) {
                val ch = myBuffer[e]
                if (ch == '\n') break
                if (ch == '\\') { e = (e + 2).coerceAtMost(myBufferEnd); continue }
                if (ch == ',' || ch == '}' || ch == ']') break
                e++
            }
            myTokenEnd = e
            val text = myBuffer.subSequence(myTokenStart, e).toString().trim()
            myTokenType = classifyScalar(text)
            // Stay in inline; next char is a separator / closer.
        }
    }

    private fun closeInline(close: IElementType, depth: Int, stack: Int) {
        myTokenEnd = myTokenStart + 1
        myTokenType = close
        val newDepth = depth - 1
        val newStack = stack shr 1
        // After a closed value the next significant char is a `,` (which sets
        // expectKey from the enclosing container) or another closer — so
        // expectKey = false here. Depth 0 means the top-level inline value is
        // finished; consume any trailing characters as a plain value.
        myState = if (newDepth <= 0) VALUE_RAW
        else encodeInline(newDepth, newStack, expectKey = false)
    }

    // -------------------------------------------------------------------
    // Token-level scanners
    // -------------------------------------------------------------------

    private fun scanIdentifier(asKey: Boolean) {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd) {
            if (isKeyChar(myBuffer[myTokenEnd])) myTokenEnd++ else break
        }
        myTokenType = if (asKey) Tokens.KEY else {
            classifyScalar(myBuffer.subSequence(myTokenStart, myTokenEnd).toString())
        }
    }

    private fun scanCommentRest() {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd && myBuffer[myTokenEnd] != '\n') myTokenEnd++
        myTokenType = Tokens.COMMENT
    }

    private fun scanHorizWhitespace() {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd) {
            val ch = myBuffer[myTokenEnd]
            if (ch == ' ' || ch == '\t') myTokenEnd++ else break
        }
        myTokenType = TokenType.WHITE_SPACE
    }

    /** Read until end of line, classifying the trimmed text when [recognise]. */
    private fun scanToEndOfLine(recognise: Boolean) {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd && myBuffer[myTokenEnd] != '\n') myTokenEnd++
        val text = myBuffer.subSequence(myTokenStart, myTokenEnd).toString().trim()
        myTokenType = if (recognise) classifyScalar(text) else Tokens.STRING_VALUE
    }

    /** Classify a scalar's surface form into a highlight token (spec 0.5.0). */
    private fun classifyScalar(text: String): IElementType = when {
        text == "true" || text == "false" -> Tokens.BOOLEAN
        text == "null" -> Tokens.NULL
        looksNumeric(text) -> if (isFloatForm(text)) Tokens.FLOAT_VALUE else Tokens.INT_VALUE
        else -> Tokens.STRING_VALUE
    }

    /**
     * Number heuristic — a faithful port of the LSP's `looks_numeric`
     * (tokens.rs): decimal / hex (`0x`) / octal (`0o`) / binary (`0b`) /
     * float (one `.` and/or one exponent), underscores allowed. A run with
     * more than one `.` — an IPv4 address (`127.0.0.1`) or a version
     * (`1.2.3`) — is NOT numeric and falls through to a string.
     */
    private fun looksNumeric(s: String): Boolean {
        if (s.isEmpty()) return false
        val rest = if (s[0] == '+' || s[0] == '-') s.substring(1) else s
        if (rest.isEmpty()) return false
        if (rest.length >= 2 && rest[0] == '0') {
            when (rest[1]) {
                'x', 'X' -> return rest.length > 2 &&
                    rest.substring(2).all { it.isDigit() || it in 'a'..'f' || it in 'A'..'F' || it == '_' }
                'o', 'O' -> return rest.length > 2 && rest.substring(2).all { it in '0'..'7' || it == '_' }
                'b', 'B' -> return rest.length > 2 && rest.substring(2).all { it == '0' || it == '1' || it == '_' }
            }
        }
        var dots = 0
        var exps = 0
        var hasDigit = false
        for (ch in rest) {
            when {
                ch.isDigit() -> hasDigit = true
                ch == '_' || ch == '+' || ch == '-' -> {}
                ch == '.' -> dots++
                ch == 'e' || ch == 'E' -> exps++
                else -> return false
            }
        }
        return hasDigit && dots <= 1 && exps <= 1
    }

    /** A numeric whose surface form is a float (decimal with `.` or exponent). */
    private fun isFloatForm(s: String): Boolean {
        val rest = if (s.isNotEmpty() && (s[0] == '+' || s[0] == '-')) s.substring(1) else s
        val lower = rest.lowercase()
        if (lower.startsWith("0x") || lower.startsWith("0o") || lower.startsWith("0b")) return false
        return rest.contains('.') || rest.contains('e') || rest.contains('E')
    }
}
