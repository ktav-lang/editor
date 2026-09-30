package lang.ktav.editor

import com.intellij.lang.ASTNode
import com.intellij.lang.folding.FoldingBuilderEx
import com.intellij.lang.folding.FoldingDescriptor
import com.intellij.openapi.editor.Document
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement
import com.intellij.psi.tree.IElementType
import lang.ktav.highlighting.KtavLexer
import lang.ktav.highlighting.KtavTokenTypes as Tokens

/** Folding uses the same classified tokens as highlighting and brace matching. */
class KtavFoldingBuilder : FoldingBuilderEx(), DumbAware {

    override fun buildFoldRegions(root: PsiElement, document: Document, quick: Boolean): Array<FoldingDescriptor> =
        foldRanges(document.charsSequence).map { FoldingDescriptor(root.node, it) }.toTypedArray()

    override fun getPlaceholderText(node: ASTNode): String {
        return when (node.text.trim().firstOrNull()) {
            '{' -> "{…}"
            '[' -> "[…]"
            '(' -> if (node.text.startsWith("((")) "((…))" else "(…)"
            else -> "…"
        }
    }

    override fun isCollapsedByDefault(node: ASTNode): Boolean = false

    companion object {
        private data class Frame(val start: Int, val end: Int, val closer: IElementType, val width: Int)

        internal fun foldRanges(text: CharSequence): List<TextRange> {
            val ranges = mutableListOf<TextRange>()
            val stack = ArrayDeque<Frame>()
            val lexer = KtavLexer()
            lexer.start(text, 0, text.length, 0)
            while (lexer.tokenType != null) {
                val type = lexer.tokenType!!
                val start = lexer.tokenStart
                val end = lexer.tokenEnd
                when (type) {
                    Tokens.LBRACE -> stack.addLast(Frame(start, end, Tokens.RBRACE, 1))
                    Tokens.LBRACKET -> stack.addLast(Frame(start, end, Tokens.RBRACKET, 1))
                    Tokens.MULTILINE_OPEN -> stack.addLast(Frame(start, end, Tokens.MULTILINE_CLOSE, end - start))
                    Tokens.RBRACE, Tokens.RBRACKET, Tokens.MULTILINE_CLOSE -> {
                        val frame = stack.lastOrNull()
                        if (frame != null && frame.closer == type && frame.width == end - start) {
                            stack.removeLast()
                            val openerLineEnd = lineTailEnd(text, frame.end)
                            if (openerLineEnd < start && KtavLexer.isLineTerminator(text[openerLineEnd]) &&
                                isAtLineStart(text, start) && isAtLineEnd(text, end)) {
                                ranges += TextRange(frame.start, end)
                            }
                        }
                    }
                }
                lexer.advance()
            }
            return ranges
        }

        // Only line boundaries are inspected here, never syntax. The lexer
        // owns comments, raw scalars, quoted keys and opaque multiline bodies.
        private fun lineTailEnd(text: CharSequence, from: Int): Int {
            var offset = from
            while (offset < text.length && KtavLexer.isHorizontalWs(text[offset])) offset++
            return offset
        }

        private fun isAtLineEnd(text: CharSequence, from: Int): Boolean {
            val end = lineTailEnd(text, from)
            return end == text.length || KtavLexer.isLineTerminator(text[end])
        }

        private fun isAtLineStart(text: CharSequence, from: Int): Boolean {
            var offset = from - 1
            while (offset >= 0 && KtavLexer.isHorizontalWs(text[offset])) offset--
            return offset < 0 || KtavLexer.isLineTerminator(text[offset])
        }
    }
}
