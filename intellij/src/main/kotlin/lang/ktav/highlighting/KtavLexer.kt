package lang.ktav.highlighting

import com.intellij.lexer.LexerBase
import com.intellij.lexer.RestartableLexer
import com.intellij.lexer.TokenIterator
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import lang.ktav.highlighting.KtavTokenTypes as Tokens

/**
 * State-machine lexer for Ktav syntax highlighting (spec 0.8.0).
 *
 * Ktav is line-oriented (`key: text`, `key:: raw`, `key.sub: {`, `## comment`).
 * Baseline (spec 0.5.0):
 *   - Comments require `##` (two hashes); a single `#` is ordinary content
 *     (allowed in keys and values).
 *   - Only `:` (string/value) and `::` (raw literal string) markers exist.
 *     Numbers are inferred from the scalar's surface form (§ 3.6 grammar),
 *     mirroring the reference parser's classifier.
 *   - Inline compounds (`{k: v, ...}`, `[v1, v2]`, nesting allowed) are
 *     tokenised structurally so their brackets become LBRACE/LBRACKET/…
 *     tokens — that is what powers brace-matching for inline objects.
 *
 * Spec 0.6.0/0.7.0/0.8.0 additions baked in here:
 *   - Keys process § 3.7 escapes; the pair separator is the first
 *     UNESCAPED `:` on the line and dotted-path segmentation splits only
 *     on UNESCAPED `.`. `\` is an escape lead inside a key: the escaped
 *     byte is folded into the same KEY token (`a\.b`, `a\:b` stay one
 *     KEY token; `path\\to`'s `\\` is a literal backslash).
 *   - A key segment may open with `"`, `'` or `` ` `` (§ 5.3.3): the
 *     quote is only structural as the FIRST code point of a segment; it
 *     runs to the next UNESCAPED same-character delimiter, and inside it
 *     `: . , { } [ ] #` are ordinary content. An unterminated quoted
 *     segment degrades gracefully (no separator found ⇒ the whole line
 *     falls back to a bare value/key run — never a crash or invalid state).
 *   - § 3.6's exact integer/float grammar (lower-case `0x`/`0o`/`0b`
 *     prefixes, single `_` only between digits) plus § 5.2's redundant-
 *     leading-zero exception decide numeric highlighting; ASCII digits
 *     only (no `Char.isDigit()` Unicode leniency).
 *   - § 3.3's exact 25-code-point whitespace set is used everywhere the
 *     lexer trims or skips whitespace, never `Char.isWhitespace()` /
 *     `String.trim()`.
 *   - A raw (`::`) value is always a String, including inside inline
 *     compounds, where a leading `{`/`[` is literal, not a nested
 *     compound. A plain inline scalar containing a recognised § 3.7
 *     escape is always a String (§ 5.2), even if the decoded body looks
 *     like a keyword or a number.
 *
 * Line states: LINE_START, AFTER_KEY, VALUE_STRING, VALUE_RAW.
 *
 * Container scopes are persistent, lexer-local nodes. Each (scope, mode)
 * has an exact state ID, so incremental convergence cannot confuse different
 * root kinds or stacks. Saved states restore directly on this lexer; a fresh
 * lexer replays the prefix once in start() to recover the unbounded context.
 * Public states describe the context BEFORE the current token.
 */
class KtavLexer : LexerBase(), RestartableLexer {

    companion object {
        private const val LINE_START = 0
        private const val AFTER_KEY = 1
        private const val VALUE_STRING = 2  // after `:` — keyword/number recognition
        private const val VALUE_RAW = 3     // after `::` — literal text, no recognition
        private const val MULTILINE_BODY_STRIPPED = 4  // inside `( ... )` — closer is a lone `)`
        private const val MULTILINE_BODY_VERBATIM = 5  // inside `(( ... ))` — closer is a lone `))`
        private const val MULTILINE_OPEN_STRIPPED = 6
        private const val MULTILINE_OPEN_VERBATIM = 7

        private const val INLINE_KEY = 16
        private const val INLINE_VALUE = 17
        private const val INLINE_RAW = 18

        private fun isInline(state: Int) = state in INLINE_KEY..INLINE_RAW

        /**
         * § 3.3 — exactly these twenty-five code points, never a host
         * `isWhitespace`/`trim()` primitive (those disagree in both
         * directions across runtimes).
         */
        private fun isKtavWhitespace(c: Char): Boolean = when (c) {
            '\u0009', '\u000A', '\u000B', '\u000C', '\u000D', '\u0020', '\u0085',
            '\u00A0', '\u1680', '\u2028', '\u2029', '\u202F', '\u205F', '\u3000' -> true
            else -> c in '\u2000'..'\u200A'
        }

        internal fun isHorizontalWs(c: Char) = c != '\n' && c != '\r' && isKtavWhitespace(c)
        internal fun isLineTerminator(c: Char) = c == '\n' || c == '\r'

        private fun CharSequence.trimKtav(): String {
            var start = 0
            var end = length
            while (start < end && isHorizontalWs(this[start])) start++
            while (end > start && isHorizontalWs(this[end - 1])) end--
            return substring(start, end)
        }

        private fun isAsciiDigit(c: Char) = c in '0'..'9'
        private fun isHexDigit(c: Char) = isAsciiDigit(c) || c in 'a'..'f' || c in 'A'..'F'
        private fun isOctDigit(c: Char) = c in '0'..'7'
        private fun isBinDigit(c: Char) = c == '0' || c == '1'

        /** `digits` matched against a single-underscore-between-digits run (§ 3.6). */
        private fun checkDigitRun(s: String, from: Int, isDigit: (Char) -> Boolean): Boolean {
            if (from >= s.length || !isDigit(s[from])) return false
            var prevUnderscore = false
            for (i in from + 1 until s.length) {
                val ch = s[i]
                if (ch == '_') {
                    if (prevUnderscore) return false
                    prevUnderscore = true
                    continue
                }
                prevUnderscore = false
                if (!isDigit(ch)) return false
            }
            return !prevUnderscore
        }

        /** § 3.6 integer literal grammar — lower-case `0x`/`0o`/`0b` prefixes only. */
        private fun matchesIntegerGrammar(s: String): Boolean {
            if (s.isEmpty()) return false
            var i = 0
            if (s[0] == '+' || s[0] == '-') i = 1
            if (i >= s.length) return false
            if (s[i] == '0' && i + 1 < s.length) {
                when (s[i + 1]) {
                    'x' -> return checkDigitRun(s, i + 2, ::isHexDigit)
                    'o' -> return checkDigitRun(s, i + 2, ::isOctDigit)
                    'b' -> return checkDigitRun(s, i + 2, ::isBinDigit)
                }
            }
            return checkDigitRun(s, i, ::isAsciiDigit)
        }

        /**
         * § 5.2 rules 13-14 exception: a base-10 digit run whose first digit
         * is `0` while at least one further digit or `_` follows (sign and
         * underscores ignored). Never true for a base-prefixed literal —
         * there the byte after `0` is `x`/`o`/`b`, not a digit/`_`.
         */
        private fun hasRedundantLeadingZero(s: String): Boolean {
            var i = 0
            if (s.isNotEmpty() && (s[0] == '+' || s[0] == '-')) i = 1
            if (i >= s.length || s[i] != '0') return false
            val next = s.getOrNull(i + 1) ?: return false
            return isAsciiDigit(next) || next == '_'
        }

        private fun scanDecPart(s: String, start: Int): Pair<Int, Boolean> {
            var i = start
            if (i >= s.length || !isAsciiDigit(s[i])) return i to false
            i++
            var prevUnderscore = false
            while (i < s.length) {
                val ch = s[i]
                if (ch == '_') {
                    if (prevUnderscore) return i to false
                    prevUnderscore = true
                    i++
                    continue
                }
                if (isAsciiDigit(ch)) {
                    prevUnderscore = false
                    i++
                    continue
                }
                break
            }
            return if (prevUnderscore) i to false else i to true
        }

        private fun scanExponent(s: String, start: Int): Pair<Int, Boolean> {
            var i = start
            if (i >= s.length || (s[i] != 'e' && s[i] != 'E')) return i to false
            i++
            if (i < s.length && (s[i] == '+' || s[i] == '-')) i++
            return scanDecPart(s, i)
        }

        /** § 3.6 float literal grammar: `d "." d exponent?` or `d exponent`. */
        private fun isFloatLiteral(s: String): Boolean {
            if (s.isEmpty()) return false
            val first = s[0]
            if (!isAsciiDigit(first) && first != '+' && first != '-') return false
            if (s.none { it == '.' || it == 'e' || it == 'E' }) return false
            var i = 0
            if (s[i] == '+' || s[i] == '-') i++
            val (afterInt, okInt) = scanDecPart(s, i)
            if (!okInt) return false
            i = afterInt
            if (i < s.length && s[i] == '.') {
                i++
                val (afterFrac, okFrac) = scanDecPart(s, i)
                if (!okFrac) return false
                i = afterFrac
                if (i < s.length && (s[i] == 'e' || s[i] == 'E')) {
                    val (afterExp, okExp) = scanExponent(s, i)
                    if (!okExp) return false
                    i = afterExp
                }
                return i == s.length
            }
            if (i < s.length && (s[i] == 'e' || s[i] == 'E')) {
                val (afterExp, okExp) = scanExponent(s, i)
                if (!okExp) return false
                i = afterExp
                return i == s.length
            }
            return false
        }

        /** § 3.7's fourteen escapes, keyed by the byte after `\`. */
        private const val RECOGNIZED_ESCAPE_CHARS = "\\,}]{[nr.:\"'`u"

        /** Raw (undecoded) text contains a `\`+recognised-escape-char pair. */
        private fun containsRecognizedEscape(text: String): Boolean {
            var i = 0
            while (i < text.length) {
                if (text[i] == '\\' && i + 1 < text.length && RECOGNIZED_ESCAPE_CHARS.indexOf(text[i + 1]) >= 0) {
                    return true
                }
                i++
            }
            return false
        }
    }

    private var myBuffer: CharSequence = ""
    private var myBufferEnd = 0
    private var myState = LINE_START
    private var myTokenState = LINE_START

    private class Scope(val rootKind: Int, val parent: Scope? = null,
                        val objectScope: Boolean = rootKind == 1, val inline: Boolean = false) {
        val states = IntArray(INLINE_RAW + 1) { -1 }
        private val children = arrayOfNulls<Scope>(4)

        fun child(objectScope: Boolean, inline: Boolean): Scope {
            val index = (if (objectScope) 1 else 0) + (if (inline) 2 else 0)
            return children[index] ?: Scope(rootKind, this, objectScope, inline).also { children[index] = it }
        }
    }

    private data class SavedState(val scope: Scope, val mode: Int)
    private val roots = Array(3) { Scope(it) }
    private var scope = roots[0]
    private val savedStates = mutableListOf(SavedState(scope, LINE_START))

    init {
        scope.states[LINE_START] = 0
    }
    private var myTokenStart = 0
    private var myTokenEnd = 0
    private var myTokenType: IElementType? = null

    override fun getStartState() = 0

    override fun isRestartableState(state: Int) = state >= 0 && state < savedStates.size

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int,
                       initialState: Int, tokenIterator: TokenIterator) {
        start(buffer, startOffset, endOffset, initialState)
    }

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
        myBuffer = buffer
        myBufferEnd = endOffset
        if (startOffset > 0 && isRestartableState(initialState)) {
            val saved = savedStates[initialState]
            scope = saved.scope
            myState = saved.mode
            myTokenEnd = startOffset
            advance()
        } else {
            // Fresh instances have no saved stack. Replay only on restart,
            // never on ordinary advance/getState. IDs remain lexer-local.
            scope = roots[0]
            myState = LINE_START
            myTokenEnd = 0
            advance()
            while (myTokenType != null && myTokenEnd <= startOffset) advance()
            myTokenStart = startOffset
        }
    }

    override fun getState() = myTokenState
    override fun getTokenType(): IElementType? = myTokenType
    override fun getTokenStart() = myTokenStart
    override fun getTokenEnd() = myTokenEnd
    override fun getBufferSequence() = myBuffer
    override fun getBufferEnd() = myBufferEnd

    override fun advance() {
        myTokenStart = myTokenEnd
        var stateId = scope.states[myState]
        if (stateId < 0) {
            stateId = savedStates.size
            savedStates.add(SavedState(scope, myState))
            scope.states[myState] = stateId
        }
        myTokenState = stateId
        if (myTokenStart >= myBufferEnd) {
            myTokenType = null
            return
        }
        advanceImpl(myBuffer[myTokenStart])
    }

    private fun advanceImpl(c: Char) {
        // CR, LF, and CRLF are equivalent line terminators; keep CRLF in one
        // whitespace token and perform the state transition only once.
        if (isLineTerminator(c)) {
            myTokenEnd = myTokenStart + 1
            if (c == '\r' && myTokenEnd < myBufferEnd && myBuffer[myTokenEnd] == '\n') {
                myTokenEnd++
            }
            myTokenType = TokenType.WHITE_SPACE
            myState = when (myState) {
                MULTILINE_OPEN_STRIPPED -> MULTILINE_BODY_STRIPPED
                MULTILINE_OPEN_VERBATIM -> MULTILINE_BODY_VERBATIM
                MULTILINE_BODY_STRIPPED, MULTILINE_BODY_VERBATIM -> myState
                else -> {
                    while (scope.inline) scope = scope.parent!!
                    LINE_START
                }
            }
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
            MULTILINE_OPEN_STRIPPED, MULTILINE_OPEN_VERBATIM -> {
                if (isHorizontalWs(c)) {
                    scanHorizWhitespace()
                } else {
                    // An incremental edit can replace the opener's whitespace tail.
                    scanToEndOfLine(recognise = false)
                    myState = LINE_START
                }
            }
            MULTILINE_BODY_STRIPPED -> scanMultilineBodyLine(closer = ")")
            MULTILINE_BODY_VERBATIM -> scanMultilineBodyLine(closer = "))")
            else -> scanLineStart(c)
        }
    }

    // -------------------------------------------------------------------
    // Line-level state handlers
    // -------------------------------------------------------------------

    private fun scanLineStart(c: Char) {
        // Only the first document code point is BOM metadata (spec 3.1).
        if (myTokenStart == 0 && c == '\uFEFF') {
            myTokenEnd++
            myTokenType = TokenType.WHITE_SPACE
            return
        }
        if (isHorizontalWs(c)) {
            scanHorizWhitespace()
            return
        }
        if (scope.rootKind == 0) {
            val separator = findLineSeparator(myTokenStart)
            val rootKind = when {
                c == '{' -> 1
                c == '[' -> 2
                separator > myTokenStart && (
                    separator + 1 == myBufferEnd ||
                        myBuffer[separator + 1] == ':' ||
                        isKtavWhitespace(myBuffer[separator + 1])
                    ) -> 1
                else -> 2
            }
            scope = roots[rootKind]
        }
        when (c) {
            '{' -> { openCompound(objectScope = true); return }
            '[' -> { openCompound(objectScope = false); return }
            '}', ']' -> if (isLineTailWhitespace(myTokenStart + 1)) {
                myTokenEnd++
                myTokenType = if (c == '}') Tokens.RBRACE else Tokens.RBRACKET
                if (scope.parent != null && scope.objectScope == (c == '}')) scope = scope.parent!!
                return
            }
        }
        // A bare (top-level or array-item) multiline block: `(`/`((` alone
        // opens a block whose body lines are opaque text until the matching
        // `)`/`))` — mirrors the `key: (`/`key: ((` case in scanValueString.
        if (c == '(') {
            if (!scanMultilineOpener()) scanToEndOfLine(recognise = true)
            return
        }
        // In Array context a pair-shaped line is a scalar, not a pair.
        if (c == ':' && myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == ':') {
            scanColonMarker()
            return
        }
        val objectScope = scope.objectScope
        if (!objectScope) {
            scanToEndOfLine(recognise = true)
            return
        }
        if (isKeyChar(c)) {
            if (findLineSeparator(myTokenStart) >= 0) {
                scanKeySegment()
                myState = AFTER_KEY
            } else {
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
            // Spec 0.6.0: an UNESCAPED `.` is the dotted-path separator.
            c == '.' -> {
                myTokenEnd++
                myTokenType = Tokens.KEY_DOT
            }
            c == ':' -> scanColonMarker()
            isHorizontalWs(c) -> scanHorizWhitespace()
            isKeyChar(c) -> scanKeySegment()
            else -> {
                myTokenEnd++
                myTokenType = Tokens.BAD_CHARACTER
            }
        }
    }

    /**
     * Char belongs to a key/identifier — everything except whitespace and
     * the structural delimiters. `#` IS a key char (a lone `#` is content;
     * only `##` at line start opens a comment). Non-ASCII letters
     * (Cyrillic / CJK / emoji) belong to keys just like ASCII.
     *
     * `\` is also a key char — it is the escape lead; `scanKeySegment`
     * handles `\x` as a two-char unit so `\.` / `\:` stay inside the KEY
     * token. A quote character (`"`, `'`, `` ` ``) is likewise an ordinary
     * key char here — § 5.3.3's positional rule (quote opens a segment
     * only as its first code point) is handled by `scanKeySegment`, not
     * by excluding quotes from this predicate.
     */
    private fun isKeyChar(c: Char): Boolean {
        if (isKtavWhitespace(c)) return false
        return when (c) {
            '{', '}', '[', ']', '(', ')', ':', '.', ',' -> false
            else -> true
        }
    }

    /**
     * Quote-and-escape-aware lookahead: does this line (starting at a fresh
     * key segment) have a real, unescaped `:` pair separator before EOL?
     * A quoted segment (§ 5.3.3) makes `: . , { } [ ]` inside it opaque; an
     * UNTERMINATED quoted segment swallows the rest of the line — including
     * any `:` in it — so this correctly reports "no separator" for it,
     * which is how the line then degrades to a bare value (§ 5.3.3 case 3).
     */
    private fun findLineSeparator(from: Int): Int {
        var i = from
        var atSegmentStart = true
        while (i < myBufferEnd) {
            val ch = myBuffer[i]
            if (isLineTerminator(ch)) return -1
            if (atSegmentStart && (ch == '"' || ch == '\'' || ch == '`')) {
                var j = i + 1
                var closed = false
                while (j < myBufferEnd) {
                    val qc = myBuffer[j]
                    if (isLineTerminator(qc)) break
                    if (qc == '\\') {
                        j += if (j + 1 < myBufferEnd && !isLineTerminator(myBuffer[j + 1])) 2 else 1
                        continue
                    }
                    if (qc == ch) { closed = true; j++; break }
                    j++
                }
                if (!closed) return -1
                i = j
                atSegmentStart = false
                continue
            }
            if (ch == '\\') {
                i += if (i + 1 < myBufferEnd && !isLineTerminator(myBuffer[i + 1])) 2 else 1
                atSegmentStart = false
                continue
            }
            if (ch == '.') { i++; atSegmentStart = true; continue }
            if (ch == ':') return i
            if (!isHorizontalWs(ch)) atSegmentStart = false
            i++
        }
        return -1
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

    /** Open a block only for `(`/`((` followed by spec whitespace and EOL/EOF. */
    private fun scanMultilineOpener(): Boolean {
        val isDouble = myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == '('
        val openerEnd = myTokenStart + if (isDouble) 2 else 1
        var e = openerEnd
        while (e < myBufferEnd && isHorizontalWs(myBuffer[e])) e++
        if (e < myBufferEnd && !isLineTerminator(myBuffer[e])) return false
        myTokenEnd = openerEnd
        myTokenType = Tokens.MULTILINE_OPEN
        // Opener-tail whitespace is not a body line.
        myState = if (isDouble) MULTILINE_OPEN_VERBATIM else MULTILINE_OPEN_STRIPPED
        return true
    }

    private fun scanValueString(c: Char, asRaw: Boolean) {
        if (isHorizontalWs(c)) {
            scanHorizWhitespace()
            return
        }
        if (!asRaw) {
            when (c) {
                '{' -> { openCompound(objectScope = true); return }
                '[' -> { openCompound(objectScope = false); return }
                '(' -> if (scanMultilineOpener()) return
            }
        }
        // Plain scalar value: whole rest of line. Raw (`::`) skips recognition.
        scanToEndOfLine(recognise = !asRaw)
    }

    private fun isLineTailWhitespace(from: Int): Boolean {
        var end = from
        while (end < myBufferEnd && isHorizontalWs(myBuffer[end])) end++
        return end == myBufferEnd || isLineTerminator(myBuffer[end])
    }

    private fun openCompound(objectScope: Boolean) {
        myTokenEnd++
        myTokenType = if (objectScope) Tokens.LBRACE else Tokens.LBRACKET
        val inline = !isLineTailWhitespace(myTokenEnd)
        scope = scope.child(objectScope, inline)
        myState = if (!inline) VALUE_RAW else if (objectScope) INLINE_KEY else INLINE_VALUE
    }

    // -------------------------------------------------------------------
    // Inline-compound tokeniser
    // -------------------------------------------------------------------

    private fun scanInline(c: Char) {
        val expectKey = myState == INLINE_KEY
        val rawPending = myState == INLINE_RAW

        if (isHorizontalWs(c)) {
            scanHorizWhitespace()
            return
        }
        when (c) {
            '{' -> if (!rawPending) {
                myTokenEnd = myTokenStart + 1
                myTokenType = Tokens.LBRACE
                scope = scope.child(objectScope = true, inline = true)
                myState = INLINE_KEY
                return
            }
            '[' -> if (!rawPending) {
                myTokenEnd = myTokenStart + 1
                myTokenType = Tokens.LBRACKET
                scope = scope.child(objectScope = false, inline = true)
                myState = INLINE_VALUE
                return
            }
            '}' -> { closeInline(Tokens.RBRACE); return }
            ']' -> { closeInline(Tokens.RBRACKET); return }
            ',' -> {
                myTokenEnd = myTokenStart + 1
                myTokenType = Tokens.COMMA
                myState = if (scope.objectScope) INLINE_KEY else INLINE_VALUE
                return
            }
            ':' -> if (expectKey) {
                // Separator right after a key: `::` raw marker or plain `:`.
                // (A `:` elsewhere — mid-value — is literal content, handled
                // by the value scan below.)
                val dbl = myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == ':'
                myTokenEnd = if (dbl) myTokenStart + 2 else myTokenStart + 1
                myTokenType = if (dbl) Tokens.DOUBLE_COLON else Tokens.COLON
                myState = if (dbl) INLINE_RAW else INLINE_VALUE
                return
            }
        }
        if (expectKey) {
            scanInlineKey()
        } else {
            // Value scalar (plain or raw): up to an unescaped `,` / `}` / `]`.
            // `{` / `[` mid-run are always literal content (head/rest rule,
            // § 5.8.5) — a value that *begins* with `{` / `[` was already
            // handled as a nested compound above when not raw-pending.
            var e = myTokenStart
            while (e < myBufferEnd) {
                val ch = myBuffer[e]
                if (isLineTerminator(ch)) break
                if (ch == '\\') {
                    e += if (e + 1 < myBufferEnd && !isLineTerminator(myBuffer[e + 1])) 2 else 1
                    continue
                }
                if (ch == ',' || ch == '}' || ch == ']') break
                e++
            }
            myTokenEnd = e
            myTokenType = if (rawPending) {
                Tokens.STRING_VALUE
            } else {
                val text = myBuffer.subSequence(myTokenStart, e).trimKtav()
                classifyScalar(text, allowEscape = true)
            }
            // Stay in inline; next char is a separator / closer.
        }
    }

    /**
     * Inline key run, quote-aware (§ 5.3.3): a quote at segment start
     * is opaque to `: . , { } [ ]` inside it. An unterminated quote
     * degrades gracefully — it swallows the rest of the line into this KEY
     * token instead of corrupting the (still-valid) inline state.
     */
    private fun scanInlineKey() {
        var e = myTokenStart
        var atSegmentStart = true
        while (e < myBufferEnd) {
            val ch = myBuffer[e]
            if (ch == '\\') {
                e += if (e + 1 < myBufferEnd && !isLineTerminator(myBuffer[e + 1])) 2 else 1
                atSegmentStart = false
                continue
            }
            if (atSegmentStart && (ch == '"' || ch == '\'' || ch == '`')) {
                var j = e + 1
                var closed = false
                while (j < myBufferEnd) {
                    val qc = myBuffer[j]
                    if (isLineTerminator(qc)) break
                    if (qc == '\\') {
                        j += if (j + 1 < myBufferEnd && !isLineTerminator(myBuffer[j + 1])) 2 else 1
                        continue
                    }
                    if (qc == ch) { closed = true; j++; break }
                    j++
                }
                if (closed) { e = j; atSegmentStart = false; continue }
                while (j < myBufferEnd && !isLineTerminator(myBuffer[j])) j++
                e = j
                break
            }
            if (isLineTerminator(ch) || ch == ':' || ch == ',' || ch == '{' || ch == '}' || ch == '[' || ch == ']') break
            if (ch == '.') atSegmentStart = true
            else if (!isHorizontalWs(ch)) atSegmentStart = false
            e++
        }
        myTokenEnd = e
        myTokenType = Tokens.KEY
        // expectKey stays true until the `:` separator flips it.
    }

    private fun closeInline(close: IElementType) {
        myTokenEnd = myTokenStart + 1
        myTokenType = close
        scope = scope.parent ?: scope
        myState = if (scope.inline) INLINE_VALUE else VALUE_RAW
    }

    // -------------------------------------------------------------------
    // Token-level scanners
    // -------------------------------------------------------------------

    /**
     * A key segment: quoted (§ 5.3.3, opened by `"`/`'`/`` ` `` as the
     * segment's first code point) or bare, with interior whitespace preserved.
     * An unterminated quote consumes the rest of the line as an ordinary KEY
     * run. Fresh line dispatch normally routes it to a scalar first.
     */
    private fun scanKeySegment() {
        val quote = myBuffer[myTokenStart]
        if (quote == '"' || quote == '\'' || quote == '`') {
            var j = myTokenStart + 1
            var closed = false
            while (j < myBufferEnd) {
                val ch = myBuffer[j]
                if (isLineTerminator(ch)) break
                if (ch == '\\') {
                    j += if (j + 1 < myBufferEnd && !isLineTerminator(myBuffer[j + 1])) 2 else 1
                    continue
                }
                if (ch == quote) { closed = true; j++; break }
                j++
            }
            if (!closed) { while (j < myBufferEnd && !isLineTerminator(myBuffer[j])) j++ }
            myTokenEnd = j
            myTokenType = Tokens.KEY
            return
        }
        // Scan a whole bare segment, including interior whitespace. A quote
        // later in that segment remains ordinary content even after a space.
        var end = myTokenStart
        while (end < myBufferEnd) {
            val ch = myBuffer[end]
            if (isLineTerminator(ch)) break
            if (ch == '\\') {
                end += if (end + 1 < myBufferEnd && !isLineTerminator(myBuffer[end + 1])) 2 else 1
                continue
            }
            if (!isHorizontalWs(ch) && !isKeyChar(ch)) break
            end++
        }
        while (end > myTokenStart && isHorizontalWs(myBuffer[end - 1])) end--
        myTokenEnd = end
        myTokenType = Tokens.KEY
    }

    private fun scanCommentRest() {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd && !isLineTerminator(myBuffer[myTokenEnd])) myTokenEnd++
        myTokenType = Tokens.COMMENT
    }

    private fun scanHorizWhitespace() {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd && isHorizontalWs(myBuffer[myTokenEnd])) myTokenEnd++
        myTokenType = TokenType.WHITE_SPACE
    }

    /** Read until end of line, classifying the trimmed text when [recognise]. */
    private fun scanToEndOfLine(recognise: Boolean) {
        myTokenEnd = myTokenStart
        while (myTokenEnd < myBufferEnd && myBuffer[myTokenEnd] != '\n' && myBuffer[myTokenEnd] != '\r') myTokenEnd++
        val text = myBuffer.subSequence(myTokenStart, myTokenEnd).trimKtav()
        myTokenType = if (recognise) classifyScalar(text) else Tokens.STRING_VALUE
    }

    /**
     * One line inside a `(...)`/`((...))` multiline block body. The body is
     * opaque text — no key/value/inline recognition — until a line whose
     * only (Ktav-whitespace-trimmed) content is exactly [closer] (`)` or
     * `))`, matching whichever opened this block). Mirrors the TextMate
     * grammar's `^.*$` body-content rule / `^\s*(\)\)?)\s*$` closer rule.
     *
     * A line that starts with leading whitespace before the closer emits
     * that whitespace as its own token first (state unchanged) so the
     * closer itself always starts a fresh token — same split top-level
     * closers already get via the `isHorizontalWs` check in [scanLineStart].
     */
    private fun scanMultilineBodyLine(closer: String) {
        var e = myTokenStart
        while (e < myBufferEnd && myBuffer[e] != '\n' && myBuffer[e] != '\r') e++
        val line = myBuffer.subSequence(myTokenStart, e).toString()
        if (line.trimKtav() == closer) {
            val closerStart = myTokenStart + line.indexOf(closer)
            if (closerStart > myTokenStart) {
                myTokenEnd = closerStart
                myTokenType = TokenType.WHITE_SPACE
                return
            }
            myTokenEnd = myTokenStart + closer.length
            myTokenType = Tokens.MULTILINE_CLOSE
            myState = LINE_START
            return
        }
        myTokenEnd = e
        myTokenType = Tokens.MULTILINE_TEXT
    }

    /**
     * Classify a scalar's surface form into a highlight token (§ 5.2).
     * [allowEscape] applies only to an inline scalar body (§ 3.7 escape
     * processing happens there, never for a multi-line pair/array-item
     * value): a recognised escape anywhere in the body forces String,
     * regardless of what the decoded text would otherwise look like.
     */
    private fun classifyScalar(text: String, allowEscape: Boolean = false): IElementType {
        if (allowEscape && containsRecognizedEscape(text)) return Tokens.STRING_VALUE
        return when {
            text == "true" || text == "false" -> Tokens.BOOLEAN
            text == "null" -> Tokens.NULL
            hasRedundantLeadingZero(text) -> Tokens.STRING_VALUE
            matchesIntegerGrammar(text) -> Tokens.INT_VALUE
            isFloatLiteral(text) -> Tokens.FLOAT_VALUE
            else -> Tokens.STRING_VALUE
        }
    }
}
