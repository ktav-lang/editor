package lang.ktav.highlighting

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.markup.EffectType
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement
import lang.ktav.KtavLanguage
import java.awt.Color
import java.awt.Font

/**
 * Highlight non-ASCII characters inside keys.
 *
 * Mirrors VS Code's `editor.unicodeHighlight` feature: any letter outside
 * basic ASCII (Cyrillic, CJK, emoji…) inside a Ktav key gets a subtle
 * boxed background so users notice non-conventional characters in
 * configuration keys (homoglyph protection / interop awareness).
 *
 * Triggered per leaf PSI element. We only act on KEY tokens, scan their
 * text, and add boxed annotations on each non-ASCII contiguous run.
 */
class KtavUnicodeAnnotator : Annotator {

    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        // Only look at our language's leaves.
        if (element.language !== KtavLanguage) return
        val node = element.node ?: return

        // Only KEY tokens — values can legitimately contain any Unicode.
        if (node.elementType != KtavTokenTypes.KEY) return

        val text = element.text
        val baseOffset = element.textRange.startOffset

        // Walk through and find runs of non-ASCII chars.
        var i = 0
        val len = text.length
        while (i < len) {
            val c = text[i]
            if (isNonAsciiVisible(c)) {
                val start = i
                // Extend run to the end of the contiguous non-ASCII span,
                // including high/low surrogates (emoji are 2 chars in UTF-16).
                while (i < len && isNonAsciiVisible(text[i])) i++
                val range = TextRange(baseOffset + start, baseOffset + i)
                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .range(range)
                    .textAttributes(NON_ASCII_KEY_ATTR)
                    .create()
            } else {
                i++
            }
        }
    }

    private fun isNonAsciiVisible(c: Char): Boolean {
        // Non-ASCII letters / symbols / surrogates. ASCII printable (0x20–0x7E)
        // and structural chars are considered "normal" and not flagged.
        return c.code !in 0x20..0x7E
    }

    companion object {
        /** Subtle red box around non-ASCII chars — matches VS Code unicode highlight. */
        private val NON_ASCII_KEY_DEFAULT_ATTRS = TextAttributes(
            null,                           // foreground: keep theme default
            null,                           // background: keep theme default
            Color(0xCC, 0x66, 0x66),       // effect color: muted red
            EffectType.BOXED,
            Font.PLAIN
        )

        val NON_ASCII_KEY_ATTR: TextAttributesKey = TextAttributesKey.createTextAttributesKey(
            "KTAV_NON_ASCII_KEY", NON_ASCII_KEY_DEFAULT_ATTRS
        )
    }
}
