package lang.ktav.highlighting

import com.intellij.lexer.LexerBase
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import lang.ktav.highlighting.KtavTokenTypes as Tokens

/**
 * Lexer for Ktav syntax.
 * Tokenizes Ktav files into syntactic elements for highlighting.
 */
class KtavLexer : LexerBase() {
    private var myBuffer: CharSequence = ""
    private var myBufferEnd = 0
    private var myState = 0
    private var myTokenStart = 0
    private var myTokenEnd = 0
    private var myTokenType: IElementType? = null

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
        myBuffer = buffer
        myBufferEnd = endOffset
        myState = initialState
        myTokenStart = startOffset
        myTokenEnd = startOffset
        advance() // Prepare the first token
    }

    override fun getState() = myState

    override fun getTokenType(): IElementType? = myTokenType

    override fun getTokenStart() = myTokenStart

    override fun getTokenEnd() = myTokenEnd

    override fun advance() {
        myTokenStart = myTokenEnd
        if (myTokenStart >= myBufferEnd) {
            myTokenType = null
            return
        }

        val c = myBuffer[myTokenStart]

        // Whitespace (use platform standard so PsiBuilder recognises it)
        if (c.isWhitespace()) {
            while (myTokenEnd < myBufferEnd && myBuffer[myTokenEnd].isWhitespace()) {
                myTokenEnd++
            }
            myTokenType = TokenType.WHITE_SPACE
            return
        }

        // Comment
        if (c == '#') {
            while (myTokenEnd < myBufferEnd && myBuffer[myTokenEnd] != '\n') {
                myTokenEnd++
            }
            myTokenType = Tokens.COMMENT
            return
        }

        // Structural characters
        when (c) {
            '{' -> {
                myTokenEnd++
                myTokenType = Tokens.LBRACE
                return
            }
            '}' -> {
                myTokenEnd++
                myTokenType = Tokens.RBRACE
                return
            }
            '[' -> {
                myTokenEnd++
                myTokenType = Tokens.LBRACKET
                return
            }
            ']' -> {
                myTokenEnd++
                myTokenType = Tokens.RBRACKET
                return
            }
            ':' -> {
                // Check for :: (double colon)
                if (myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1] == ':') {
                    myTokenEnd = myTokenStart + 2
                    myTokenType = Tokens.DOUBLE_COLON
                } else {
                    myTokenEnd = myTokenStart + 1
                    myTokenType = Tokens.COLON
                }
                return
            }
        }

        // String in quotes
        if (c == '"' || c == '\'') {
            scanString(c)
            return
        }

        // Number
        if (c.isDigit() || (c == '-' && myTokenStart + 1 < myBufferEnd && myBuffer[myTokenStart + 1].isDigit())) {
            scanNumber()
            return
        }

        // Keywords and identifiers (with optional dotted segments: foo.bar.baz)
        if (c.isLetter() || c == '_') {
            scanIdentifierOrKeyword()
            return
        }
        // Dot at unexpected position — treat as identifier-extender if surrounded by ident chars,
        // otherwise as bad char. Standalone dot is bad in Ktav.
        if (c == '.') {
            // Check if this is part of dotted key path: previous and next are ident chars
            val prevOk = myTokenStart > 0 &&
                (myBuffer[myTokenStart - 1].isLetterOrDigit() || myBuffer[myTokenStart - 1] == '_')
            val nextOk = myTokenStart + 1 < myBufferEnd &&
                (myBuffer[myTokenStart + 1].isLetter() || myBuffer[myTokenStart + 1] == '_')
            if (prevOk && nextOk) {
                myTokenEnd = myTokenStart + 1
                myTokenType = Tokens.KEY
                return
            }
        }

        // Bad character
        myTokenEnd++
        myTokenType = Tokens.BAD_CHARACTER
    }

    private fun scanString(quote: Char) {
        myTokenEnd = myTokenStart + 1
        while (myTokenEnd < myBufferEnd) {
            val c = myBuffer[myTokenEnd]
            if (c == quote) {
                myTokenEnd++
                break
            }
            if (c == '\\' && myTokenEnd + 1 < myBufferEnd) {
                myTokenEnd += 2
            } else {
                myTokenEnd++
            }
        }
        myTokenType = Tokens.STRING
    }

    private fun scanNumber() {
        myTokenEnd = myTokenStart
        if (myBuffer[myTokenEnd] == '-') {
            myTokenEnd++
        }
        while (myTokenEnd < myBufferEnd && (myBuffer[myTokenEnd].isDigit() || myBuffer[myTokenEnd] == '.')) {
            myTokenEnd++
        }
        myTokenType = Tokens.NUMBER
    }

    private fun scanIdentifierOrKeyword() {
        myTokenEnd = myTokenStart
        // Identifier characters: letter, digit, underscore. Dots are separate
        // tokens and handled via advance() so we don't grab them here — but
        // they bridge ident segments without becoming bad chars (see advance()).
        while (myTokenEnd < myBufferEnd && (myBuffer[myTokenEnd].isLetterOrDigit() || myBuffer[myTokenEnd] == '_')) {
            myTokenEnd++
        }

        val text = myBuffer.subSequence(myTokenStart, myTokenEnd).toString()
        myTokenType = when (text) {
            "true", "false" -> Tokens.BOOLEAN
            "null" -> Tokens.NULL
            else -> Tokens.KEY  // identifier — most often a key name
        }
    }

    override fun getBufferSequence() = myBuffer

    override fun getBufferEnd() = myBufferEnd
}
