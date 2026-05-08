package lang.ktav.highlighting

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.markup.EffectType
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement
import lang.ktav.KtavLanguage
import java.awt.Color
import java.awt.Font

/**
 * Highlight characters of a foreign script when a key mixes scripts.
 *
 * Mirrors VS Code's `editor.unicodeHighlight.ambiguousCharacters` /
 * `unicodeHighlight.invisibleCharacters` philosophy: a Latin-only key
 * (`my_key`) and a Cyrillic-only key (`имя_ключа`) are both legitimate
 * — neither is flagged. What's dangerous is the mix: `аdmin` (where
 * `а` is Cyrillic but visually identical to Latin `a`) silently bypasses
 * key-equality checks. So we only highlight when a key contains BOTH
 * ASCII letters AND non-ASCII letters, and we paint only the
 * minority-script characters as the visual cue.
 *
 * Digits, `_`, `-` are script-neutral and don't trigger the mix flag —
 * `сервер2` (Cyrillic + ASCII digit) stays unhighlighted.
 *
 * Triggered per leaf PSI element. Acts only on KEY tokens; values can
 * legitimately contain any text. Dotted-key segments are separate KEY
 * tokens, so `a.б` is two single-script segments — not flagged.
 */
class KtavUnicodeAnnotator : Annotator {

    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        if (element.language !== KtavLanguage) return
        val node = element.node ?: return
        if (node.elementType != KtavTokenTypes.KEY) return

        val text = element.text
        val baseOffset = element.textRange.startOffset

        // Classify the key in one pass: do we have ASCII letters? Do we
        // have non-ASCII letters? Anything else (digits, `_`, `-`, etc.)
        // is script-neutral and ignored for the mix check.
        var hasAsciiLetter = false
        var hasNonAsciiLetter = false
        for (c in text) {
            if (isAsciiLetter(c)) {
                hasAsciiLetter = true
            } else if (isNonAsciiLetter(c)) {
                hasNonAsciiLetter = true
            }
            if (hasAsciiLetter && hasNonAsciiLetter) break
        }
        if (!(hasAsciiLetter && hasNonAsciiLetter)) {
            // Pure single-script (or no letters at all) — no warning.
            return
        }

        // Mixed: highlight every contiguous run of non-ASCII letters.
        // Surrogate pairs (e.g. emoji) are 2 chars in UTF-16; we extend
        // the run while `isNonAsciiLetter` keeps matching, which covers
        // both code units in pair.
        var i = 0
        val len = text.length
        while (i < len) {
            if (isNonAsciiLetter(text[i])) {
                val start = i
                while (i < len && isNonAsciiLetter(text[i])) i++
                val range = TextRange(baseOffset + start, baseOffset + i)
                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .range(range)
                    .textAttributes(MIXED_SCRIPT_ATTR)
                    .create()
            } else {
                i++
            }
        }
    }

    /** ASCII A–Z / a–z. Digits and underscore deliberately excluded. */
    private fun isAsciiLetter(c: Char): Boolean {
        val code = c.code
        return code in 0x41..0x5A || code in 0x61..0x7A
    }

    /** Any letter outside ASCII A–Z / a–z range. Digits, `_`, `-`, `.`,
     *  surrogate halves, etc. excluded — those are script-neutral. */
    private fun isNonAsciiLetter(c: Char): Boolean {
        val code = c.code
        if (code <= 0x7F) return false
        // Java's Character#isLetter is Unicode-aware. High surrogates
        // (0xD800–0xDBFF) and low surrogates (0xDC00–0xDFFF) aren't
        // letters in isolation, but `c.isLetter()` still returns false
        // for them — emoji surrogate pairs typically fail the check
        // here. That's fine: emoji are symbols, not letters, and a key
        // mixing emoji with ASCII (e.g. `🔧config`) is intentional
        // decoration, not a homoglyph attack.
        return c.isLetter()
    }

    companion object {
        /** Muted red box around mixed-script characters. */
        private val MIXED_SCRIPT_DEFAULT_ATTRS = TextAttributes(
            null,                       // foreground: theme default
            null,                       // background: theme default
            Color(0xCC, 0x66, 0x66),    // effect color: muted red
            EffectType.BOXED,
            Font.PLAIN,
        )

        val MIXED_SCRIPT_ATTR: TextAttributesKey = TextAttributesKey.createTextAttributesKey(
            "KTAV_MIXED_SCRIPT_KEY", MIXED_SCRIPT_DEFAULT_ATTRS,
        )
    }
}
