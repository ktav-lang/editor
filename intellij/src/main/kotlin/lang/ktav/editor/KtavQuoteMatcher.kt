package lang.ktav.editor

import com.intellij.openapi.editor.colors.CodeInsightColors
import com.intellij.openapi.Disposable
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.editor.event.CaretEvent
import com.intellij.openapi.editor.event.CaretListener
import com.intellij.openapi.editor.event.EditorFactoryEvent
import com.intellij.openapi.editor.event.EditorFactoryListener
import com.intellij.openapi.editor.ex.EditorEx
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.HighlighterTargetArea
import com.intellij.openapi.editor.markup.RangeHighlighter
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.util.Key
import lang.ktav.KtavFileType
import lang.ktav.highlighting.KtavQuotedKeyAnnotator
import lang.ktav.highlighting.KtavTokenTypes

/**
 * Highlights the paired quote of a quoted key segment the way the platform
 * highlights a matching brace: caret right before or after either quote.
 * A quoted segment is one KEY token, so the brace matcher cannot pair it.
 */
class KtavQuoteMatcher : EditorFactoryListener {

    override fun editorCreated(event: EditorFactoryEvent) {
        val editor = event.editor as? EditorEx ?: return
        val file = FileDocumentManager.getInstance().getFile(editor.document) ?: return
        if (file.fileType != KtavFileType) return
        val disposable = Disposer.newDisposable("ktav quote matcher")
        editor.putUserData(DISPOSABLE, disposable)
        editor.caretModel.addCaretListener(object : CaretListener {
            override fun caretPositionChanged(event: CaretEvent) = update(editor)
        }, disposable)
    }

    override fun editorReleased(event: EditorFactoryEvent) {
        val editor = event.editor
        clear(editor)
        editor.getUserData(DISPOSABLE)?.let(Disposer::dispose)
        editor.putUserData(DISPOSABLE, null)
    }

    private fun update(editor: EditorEx) {
        clear(editor)
        if (editor.caretModel.caretCount != 1 || editor.selectionModel.hasSelection()) return
        val caret = editor.caretModel.offset
        val chars = editor.document.charsSequence
        val pair = sequenceOf(caret, caret - 1)
            .filter { it in 0 until chars.length }
            .mapNotNull { at ->
                val it = editor.highlighter.createIterator(at)
                if (it.atEnd() || it.tokenType != KtavTokenTypes.KEY) return@mapNotNull null
                quotePairAt(it.start, chars.subSequence(it.start, it.end), caret)
            }
            .firstOrNull() ?: return
        val markup = editor.markupModel
        editor.putUserData(HIGHLIGHTERS, pair.map { offset ->
            markup.addRangeHighlighter(
                CodeInsightColors.MATCHED_BRACE_ATTRIBUTES, offset, offset + 1,
                HighlighterLayer.LAST + 1, HighlighterTargetArea.EXACT_RANGE,
            )
        })
    }

    private fun clear(editor: Editor) {
        editor.getUserData(HIGHLIGHTERS)?.forEach { editor.markupModel.removeHighlighter(it) }
        editor.putUserData(HIGHLIGHTERS, null)
    }

    companion object {
        private val HIGHLIGHTERS = Key.create<List<RangeHighlighter>>("ktav.quote.match")
        private val DISPOSABLE = Key.create<Disposable>("ktav.quote.match.disposable")

        /**
         * Offsets of both quotes when [caret] touches a quote of the quoted KEY
         * token starting at [tokenStart], else null.
         */
        fun quotePairAt(tokenStart: Int, tokenText: CharSequence, caret: Int): List<Int>? {
            if (!KtavQuotedKeyAnnotator.isQuotedSegment(tokenText)) return null
            val open = tokenStart
            val close = tokenStart + tokenText.length - 1
            return if (caret == open || caret == open + 1 || caret == close || caret == close + 1) {
                listOf(open, close)
            } else {
                null
            }
        }
    }
}
