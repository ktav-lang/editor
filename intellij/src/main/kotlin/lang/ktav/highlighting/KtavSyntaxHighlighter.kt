package lang.ktav.highlighting

import com.intellij.lexer.Lexer
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.HighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.fileTypes.SyntaxHighlighterFactory
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
        // Color keys
        private val KEY_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_KEY",
            DefaultLanguageHighlighterColors.INSTANCE_FIELD
        )

        private val STRING_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_STRING",
            DefaultLanguageHighlighterColors.STRING
        )

        private val NUMBER_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_NUMBER",
            DefaultLanguageHighlighterColors.NUMBER
        )

        private val BOOLEAN_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_BOOLEAN",
            DefaultLanguageHighlighterColors.KEYWORD
        )

        private val NULL_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_NULL",
            DefaultLanguageHighlighterColors.KEYWORD
        )

        private val BRACES_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_BRACES",
            DefaultLanguageHighlighterColors.BRACES
        )

        private val BRACKETS_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_BRACKETS",
            DefaultLanguageHighlighterColors.BRACKETS
        )

        private val COLON_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_COLON",
            DefaultLanguageHighlighterColors.OPERATION_SIGN
        )

        private val COMMENT_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_COMMENT",
            DefaultLanguageHighlighterColors.LINE_COMMENT
        )

        private val BAD_CHAR_ATTR = TextAttributesKey.createTextAttributesKey(
            "KTAV_BAD_CHARACTER",
            HighlighterColors.BAD_CHARACTER
        )

        private val EMPTY_ATTRS = emptyArray<TextAttributesKey>()

        private val ATTRIBUTES = mapOf(
            Tokens.KEY to arrayOf(KEY_ATTR),
            Tokens.STRING to arrayOf(STRING_ATTR),
            Tokens.NUMBER to arrayOf(NUMBER_ATTR),
            Tokens.BOOLEAN to arrayOf(BOOLEAN_ATTR),
            Tokens.NULL to arrayOf(NULL_ATTR),
            Tokens.LBRACE to arrayOf(BRACES_ATTR),
            Tokens.RBRACE to arrayOf(BRACES_ATTR),
            Tokens.LBRACKET to arrayOf(BRACKETS_ATTR),
            Tokens.RBRACKET to arrayOf(BRACKETS_ATTR),
            Tokens.COLON to arrayOf(COLON_ATTR),
            Tokens.DOUBLE_COLON to arrayOf(COLON_ATTR),
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
        val attrs = ATTRIBUTES[tokenType] ?: EMPTY_ATTRS
        if (tokenType != null) {
            log.debug("KtavSyntaxHighlighter.getTokenHighlights($tokenType) -> ${attrs.size} attributes")
        }
        return attrs
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
