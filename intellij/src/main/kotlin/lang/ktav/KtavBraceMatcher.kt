package lang.ktav

import com.intellij.lang.BracePair
import com.intellij.lang.PairedBraceMatcher
import com.intellij.psi.PsiFile
import com.intellij.psi.tree.IElementType
import lang.ktav.highlighting.KtavTokenTypes

/**
 * Brace matching + auto-closing for Ktav.
 *
 * IntelliJ uses [PairedBraceMatcher] for two related features:
 *   1. Highlight matching brace when the caret is next to the opener
 *      or closer (`{` highlights `}`, etc.).
 *   2. Auto-close on type: when the user types an opener, IDE inserts
 *      the matching closer and places the caret between them.
 *
 * Pairs registered:
 *   - `{` ↔ `}`     (object)
 *   - `[` ↔ `]`     (array)
 *   - `(` ↔ `)`     (multi-line stripped) — also covers `((` ↔ `))` since
 *                   the lexer emits MULTILINE_OPEN/CLOSE for both forms.
 *
 * The `structural` flag = true marks pairs that contribute to the parser's
 * brace-balancing — it lets IDE auto-indent the next line after the opener.
 */
class KtavBraceMatcher : PairedBraceMatcher {

    override fun getPairs(): Array<BracePair> = PAIRS

    override fun isPairedBracesAllowedBeforeType(
        lbraceType: IElementType,
        contextType: IElementType?
    ): Boolean = true

    override fun getCodeConstructStart(file: PsiFile?, openingBraceOffset: Int): Int =
        openingBraceOffset

    companion object {
        private val PAIRS = arrayOf(
            BracePair(KtavTokenTypes.LBRACE, KtavTokenTypes.RBRACE, true),
            BracePair(KtavTokenTypes.LBRACKET, KtavTokenTypes.RBRACKET, true),
            BracePair(KtavTokenTypes.MULTILINE_OPEN, KtavTokenTypes.MULTILINE_CLOSE, true),
        )
    }
}
