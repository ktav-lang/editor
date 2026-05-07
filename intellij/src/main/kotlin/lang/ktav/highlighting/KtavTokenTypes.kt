package lang.ktav.highlighting

import com.intellij.psi.tree.IElementType
import lang.ktav.KtavLanguage

/**
 * Token types for Ktav syntax highlighting.
 * These represent the different syntactic elements of the Ktav language.
 */
object KtavTokenTypes {
    // Keys (left side of `:`)
    val KEY = IElementType("KTAV_KEY", KtavLanguage)
    val KEY_DOT = IElementType("KTAV_KEY_DOT", KtavLanguage)  // dot in dotted key paths

    // Type markers (act like keywords for the type system)
    val MARKER_INT = IElementType("KTAV_MARKER_INT", KtavLanguage)      // :i
    val MARKER_FLOAT = IElementType("KTAV_MARKER_FLOAT", KtavLanguage)  // :f
    val COLON = IElementType("KTAV_COLON", KtavLanguage)                // :  (string value)
    val DOUBLE_COLON = IElementType("KTAV_DOUBLE_COLON", KtavLanguage)  // :: (raw string)

    // Values
    val STRING_VALUE = IElementType("KTAV_STRING_VALUE", KtavLanguage)  // value after `:` or `::`
    val INT_VALUE = IElementType("KTAV_INT_VALUE", KtavLanguage)        // value after `:i`
    val FLOAT_VALUE = IElementType("KTAV_FLOAT_VALUE", KtavLanguage)    // value after `:f`
    val BOOLEAN = IElementType("KTAV_BOOLEAN", KtavLanguage)
    val NULL = IElementType("KTAV_NULL", KtavLanguage)

    // Multi-line markers and content
    val MULTILINE_OPEN = IElementType("KTAV_MULTILINE_OPEN", KtavLanguage)    // ( or ((
    val MULTILINE_CLOSE = IElementType("KTAV_MULTILINE_CLOSE", KtavLanguage)  // ) or ))
    val MULTILINE_TEXT = IElementType("KTAV_MULTILINE_TEXT", KtavLanguage)

    // Structural
    val LBRACE = IElementType("KTAV_LBRACE", KtavLanguage)
    val RBRACE = IElementType("KTAV_RBRACE", KtavLanguage)
    val LBRACKET = IElementType("KTAV_LBRACKET", KtavLanguage)
    val RBRACKET = IElementType("KTAV_RBRACKET", KtavLanguage)

    // Backwards-compat aliases for any old code paths still referencing
    // the original names. Map them to the new value tokens.
    @Deprecated("use STRING_VALUE")
    val STRING = STRING_VALUE
    @Deprecated("use INT_VALUE / FLOAT_VALUE")
    val NUMBER = INT_VALUE

    // Comments
    val COMMENT = IElementType("KTAV_COMMENT", KtavLanguage)

    // Whitespace and errors
    val WHITESPACE = IElementType("KTAV_WHITESPACE", KtavLanguage)
    val BAD_CHARACTER = IElementType("KTAV_BAD_CHARACTER", KtavLanguage)
}
