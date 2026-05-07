package lang.ktav

import com.intellij.lang.ASTNode
import com.intellij.lang.folding.FoldingBuilderEx
import com.intellij.lang.folding.FoldingDescriptor
import com.intellij.openapi.editor.Document
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement

/**
 * Code folding for Ktav.
 *
 * Folds matching pairs of `{` `}`, `[` `]`, `(` `)`, `((` `))` whose
 * opening token is at end-of-line and the matching closing token starts
 * a later line. The lexer doesn't build a structured PSI tree, so we
 * scan the document text directly — same approach IDEA uses for plain
 * text and properties files.
 *
 * Implements [DumbAware] so folding works during indexing.
 */
class KtavFoldingBuilder : FoldingBuilderEx(), DumbAware {

    override fun buildFoldRegions(root: PsiElement, document: Document, quick: Boolean): Array<FoldingDescriptor> {
        val text = document.text
        val descriptors = mutableListOf<FoldingDescriptor>()

        // Stack frames: (openOffset, kind). Kind is the closer string.
        data class Frame(val openOffset: Int, val closer: String)
        val stack = ArrayDeque<Frame>()

        var i = 0
        val len = text.length
        while (i < len) {
            // Skip everything until we hit interesting bracket-like char.
            val c = text[i]

            // Comments — skip to end of line. (Comment is `^\s*#…`, but
            // a `#` mid-line also begins inline comment per spec; either
            // way we don't match brackets inside.)
            if (c == '#' && isAtLineStart(text, i)) {
                i = endOfLine(text, i)
                continue
            }

            when (c) {
                '{' -> if (isOpenAtLineEnd(text, i)) {
                    stack.addLast(Frame(i, "}"))
                }
                '[' -> if (isOpenAtLineEnd(text, i)) {
                    stack.addLast(Frame(i, "]"))
                }
                '(' -> {
                    // Detect `((` opener at line end; otherwise single `(`.
                    val isDouble = i + 1 < len && text[i + 1] == '('
                    val tokLen = if (isDouble) 2 else 1
                    if (isOpenAtLineEndAt(text, i, tokLen)) {
                        stack.addLast(Frame(i, if (isDouble) "))" else ")"))
                    }
                    i += tokLen - 1 // consume extra char for `((`
                }
                '}', ']' -> {
                    if (isCloseAtLineStart(text, i)) {
                        val closer = c.toString()
                        val frame = stack.lastOrNull()
                        if (frame != null && frame.closer == closer) {
                            stack.removeLast()
                            addDescriptor(descriptors, frame.openOffset, i + 1, root.node)
                        }
                    }
                }
                ')' -> {
                    val isDouble = i + 1 < len && text[i + 1] == ')'
                    val closer = if (isDouble) "))" else ")"
                    val tokLen = if (isDouble) 2 else 1
                    if (isCloseAtLineStartAt(text, i, tokLen)) {
                        val frame = stack.lastOrNull()
                        if (frame != null && frame.closer == closer) {
                            stack.removeLast()
                            addDescriptor(descriptors, frame.openOffset, i + tokLen, root.node)
                        }
                    }
                    i += tokLen - 1
                }
            }
            i++
        }

        return descriptors.toTypedArray()
    }

    override fun getPlaceholderText(node: ASTNode): String {
        // Show ellipsis matching the opener so users still see structure
        // when collapsed.
        return when (node.text.trim().firstOrNull()) {
            '{' -> "{…}"
            '[' -> "[…]"
            '(' -> if (node.text.startsWith("((")) "((…))" else "(…)"
            else -> "…"
        }
    }

    override fun isCollapsedByDefault(node: ASTNode): Boolean = false

    // ---------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------

    private fun addDescriptor(out: MutableList<FoldingDescriptor>, start: Int, end: Int, node: ASTNode) {
        if (end - start < 2) return
        out += FoldingDescriptor(node, TextRange(start, end))
    }

    private fun isAtLineStart(text: String, offset: Int): Boolean {
        var j = offset - 1
        while (j >= 0) {
            val c = text[j]
            if (c == '\n') return true
            if (c != ' ' && c != '\t') return false
            j--
        }
        return true
    }

    private fun isOpenAtLineEnd(text: String, offset: Int): Boolean =
        isOpenAtLineEndAt(text, offset, 1)

    private fun isOpenAtLineEndAt(text: String, offset: Int, tokLen: Int): Boolean {
        // After opener (and trailing whitespace), expect newline or EOF.
        var j = offset + tokLen
        while (j < text.length) {
            val c = text[j]
            if (c == '\n') return true
            if (c != ' ' && c != '\t') return false
            j++
        }
        return true
    }

    private fun isCloseAtLineStart(text: String, offset: Int): Boolean =
        isCloseAtLineStartAt(text, offset, 1)

    private fun isCloseAtLineStartAt(text: String, offset: Int, tokLen: Int): Boolean {
        // Before closer, only whitespace from line start.
        if (!isAtLineStart(text, offset)) return false
        // After closer, only whitespace until newline.
        var j = offset + tokLen
        while (j < text.length) {
            val c = text[j]
            if (c == '\n') return true
            if (c != ' ' && c != '\t') return false
            j++
        }
        return true
    }

    private fun endOfLine(text: String, offset: Int): Int {
        var j = offset
        while (j < text.length && text[j] != '\n') j++
        return j
    }
}
