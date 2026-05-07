package lang.ktav.highlighting

import com.intellij.psi.tree.IElementType
import lang.ktav.KtavLanguage

/**
 * Token types for Ktav syntax highlighting.
 * These represent the different syntactic elements of the Ktav language.
 */
object KtavTokenTypes {
    // Literals
    val KEY = IElementType("KTAV_KEY", KtavLanguage)
    val STRING = IElementType("KTAV_STRING", KtavLanguage)
    val NUMBER = IElementType("KTAV_NUMBER", KtavLanguage)
    val BOOLEAN = IElementType("KTAV_BOOLEAN", KtavLanguage)
    val NULL = IElementType("KTAV_NULL", KtavLanguage)

    // Structural
    val LBRACE = IElementType("KTAV_LBRACE", KtavLanguage)
    val RBRACE = IElementType("KTAV_RBRACE", KtavLanguage)
    val LBRACKET = IElementType("KTAV_LBRACKET", KtavLanguage)
    val RBRACKET = IElementType("KTAV_RBRACKET", KtavLanguage)
    val COLON = IElementType("KTAV_COLON", KtavLanguage)
    val DOUBLE_COLON = IElementType("KTAV_DOUBLE_COLON", KtavLanguage)

    // Comments
    val COMMENT = IElementType("KTAV_COMMENT", KtavLanguage)

    // Whitespace and errors
    val WHITESPACE = IElementType("KTAV_WHITESPACE", KtavLanguage)
    val BAD_CHARACTER = IElementType("KTAV_BAD_CHARACTER", KtavLanguage)
}
