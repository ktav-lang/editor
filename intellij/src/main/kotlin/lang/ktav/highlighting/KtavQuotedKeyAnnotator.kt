package lang.ktav.highlighting

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement
import lang.ktav.KtavLanguage

/**
 * Colours a quoted key segment (§ 5.3.3) apart from a bare one: the paired
 * delimiters get [KEY_QUOTE], the content [QUOTED_KEY]. The lexer emits a
 * quoted segment as one KEY token, delimiters included.
 */
class KtavQuotedKeyAnnotator : Annotator {

    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        if (element.language !== KtavLanguage) return
        if (element.node?.elementType != KtavTokenTypes.KEY) return
        val text = element.text
        if (!isQuotedSegment(text)) return
        val start = element.textRange.startOffset
        val end = element.textRange.endOffset
        mark(holder, TextRange(start, start + 1), KEY_QUOTE)
        if (text.length > 2) mark(holder, TextRange(start + 1, end - 1), QUOTED_KEY)
        mark(holder, TextRange(end - 1, end), KEY_QUOTE)
    }

    private fun mark(holder: AnnotationHolder, range: TextRange, key: TextAttributesKey) {
        holder.newSilentAnnotation(HighlightSeverity.INFORMATION).range(range).textAttributes(key).create()
    }

    companion object {
        val KEY_QUOTE: TextAttributesKey = TextAttributesKey.createTextAttributesKey(
            "KTAV_KEY_QUOTE", DefaultLanguageHighlighterColors.KEYWORD,
        )
        val QUOTED_KEY: TextAttributesKey = TextAttributesKey.createTextAttributesKey(
            "KTAV_QUOTED_KEY", DefaultLanguageHighlighterColors.INSTANCE_FIELD,
        )

        /** A KEY token that is a whole quoted segment: `"…"`, `'…'` or `` `…` ``. */
        fun isQuotedSegment(text: CharSequence): Boolean =
            text.length >= 2 && text[0] in QUOTES && text[text.length - 1] == text[0]

        private val QUOTES = charArrayOf('"', '\'', '`')
    }
}
