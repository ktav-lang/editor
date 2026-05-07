package lang.ktav.highlighting

import com.intellij.lexer.Lexer
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.HighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.fileTypes.SyntaxHighlighterFactory
import java.awt.Color
import java.awt.Font
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.tree.IElementType
import lang.ktav.KtavFileType
import lang.ktav.KtavLanguage
import lang.ktav.highlighting.KtavTokenTypes as Tokens

/**
 * Syntax highlighter for Ktav files using native IntelliJ highlighting.
 * Provides color schemes for different Ktav syntax elements.
 */
class KtavSyntaxHighlighter : SyntaxHighlighter {
    companion object {
        private val log = com.intellij.openapi.diagnostic.Logger.getInstance(KtavSyntaxHighlighter::class.java)

        init {
            log.info("KtavSyntaxHighlighter initialized")
        }
        // Keys / metadata — explicit hard-coded colour so the result is
        // visible regardless of the active color scheme. Orange foreground
        // matches the VS Code "entity.name.tag" tone (used for HTML tags
        // and our equivalent — Ktav keys).
        private val KEY_DEFAULT_ATTRS = TextAttributes(
            Color(0xCC7832), // foreground: orange
            null,            // background: theme default
            null,            // effect color
            null,            // effect type
            Font.PLAIN
        )
        private val KEY_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_KEY", KEY_DEFAULT_ATTRS
        )
        private val KEY_DOT_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_KEY_DOT", DefaultLanguageHighlighterColors.DOT
        )

        // Type markers (`:i`, `:f`, `::`) — keyword-style colour
        private val MARKER_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_MARKER", DefaultLanguageHighlighterColors.METADATA
        )

        // Plain `:` separator — operator colour
        private val COLON_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_COLON", DefaultLanguageHighlighterColors.OPERATION_SIGN
        )

        // Values
        private val STRING_VALUE_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_STRING_VALUE", DefaultLanguageHighlighterColors.STRING
        )
        private val INT_VALUE_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_INT_VALUE", DefaultLanguageHighlighterColors.NUMBER
        )
        // CONSTANT colour for float — visually distinct from NUMBER
        // (typical schemes give it a different hue: lighter / teal / italic).
        private val FLOAT_VALUE_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_FLOAT_VALUE", DefaultLanguageHighlighterColors.CONSTANT
        )
        private val BOOLEAN_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_BOOLEAN", DefaultLanguageHighlighterColors.KEYWORD
        )
        private val NULL_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_NULL", DefaultLanguageHighlighterColors.KEYWORD
        )

        // Multi-line text (verbatim/stripped)
        private val MULTILINE_BRACKET_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_MULTILINE_BRACKET", DefaultLanguageHighlighterColors.PARENTHESES
        )
        private val MULTILINE_TEXT_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_MULTILINE_TEXT", DefaultLanguageHighlighterColors.STRING
        )

        // Structural
        private val BRACES_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_BRACES", DefaultLanguageHighlighterColors.BRACES
        )
        private val BRACKETS_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_BRACKETS", DefaultLanguageHighlighterColors.BRACKETS
        )

        // Comments + errors
        private val COMMENT_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_COMMENT", DefaultLanguageHighlighterColors.LINE_COMMENT
        )
        private val BAD_CHAR_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_BAD_CHARACTER", HighlighterColors.BAD_CHARACTER
        )

        private val EMPTY_ATTRS = emptyArray<TextAttributesKey>()

        private val ATTRIBUTES = mapOf(
            // Keys
            Tokens.KEY to arrayOf(KEY_ATTR),
            Tokens.KEY_DOT to arrayOf(KEY_DOT_ATTR),

            // Markers + separator
            Tokens.MARKER_INT to arrayOf(MARKER_ATTR),
            Tokens.MARKER_FLOAT to arrayOf(MARKER_ATTR),
            Tokens.DOUBLE_COLON to arrayOf(MARKER_ATTR),
            Tokens.COLON to arrayOf(COLON_ATTR),

            // Values
            Tokens.STRING_VALUE to arrayOf(STRING_VALUE_ATTR),
            Tokens.INT_VALUE to arrayOf(INT_VALUE_ATTR),
            Tokens.FLOAT_VALUE to arrayOf(FLOAT_VALUE_ATTR),
            Tokens.BOOLEAN to arrayOf(BOOLEAN_ATTR),
            Tokens.NULL to arrayOf(NULL_ATTR),

            // Multi-line
            Tokens.MULTILINE_OPEN to arrayOf(MULTILINE_BRACKET_ATTR),
            Tokens.MULTILINE_CLOSE to arrayOf(MULTILINE_BRACKET_ATTR),
            Tokens.MULTILINE_TEXT to arrayOf(MULTILINE_TEXT_ATTR),

            // Structural
            Tokens.LBRACE to arrayOf(BRACES_ATTR),
            Tokens.RBRACE to arrayOf(BRACES_ATTR),
            Tokens.LBRACKET to arrayOf(BRACKETS_ATTR),
            Tokens.RBRACKET to arrayOf(BRACKETS_ATTR),

            // Comments + errors
            Tokens.COMMENT to arrayOf(COMMENT_ATTR),
            Tokens.BAD_CHARACTER to arrayOf(BAD_CHAR_ATTR),
        )
    }

    private val lexer = KtavLexer()

    override fun getHighlightingLexer(): Lexer {
        log.info("KtavSyntaxHighlighter.getHighlightingLexer() called")
        return lexer
    }

    override fun getTokenHighlights(tokenType: IElementType?): Array<TextAttributesKey> {
        return ATTRIBUTES[tokenType] ?: EMPTY_ATTRS
    }
}

/**
 * Factory for creating KtavSyntaxHighlighter instances.
 */
class KtavSyntaxHighlighterFactory : SyntaxHighlighterFactory() {
    companion object {
        private val log = com.intellij.openapi.diagnostic.Logger.getInstance(KtavSyntaxHighlighterFactory::class.java)

        init {
            println(">>> KtavSyntaxHighlighterFactory class loaded!")
            log.info("KtavSyntaxHighlighterFactory initialized")
        }
    }

    init {
        println(">>> KtavSyntaxHighlighterFactory instance created!")
    }

    override fun getSyntaxHighlighter(project: Project?, virtualFile: VirtualFile?): SyntaxHighlighter {
        val msg = "KtavSyntaxHighlighterFactory.getSyntaxHighlighter() for: ${virtualFile?.path}"
        println(">>> $msg")
        log.info(msg)
        return KtavSyntaxHighlighter()
    }
}
