package lang.ktav.lsp.diagnostics

import com.intellij.codeInsight.daemon.DaemonCodeAnalyzer
import com.intellij.openapi.Disposable
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Document
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.editor.EditorFactory
import com.intellij.openapi.editor.colors.CodeInsightColors
import com.intellij.openapi.editor.event.EditorFactoryEvent
import com.intellij.openapi.editor.event.EditorFactoryListener
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.HighlighterTargetArea
import com.intellij.openapi.editor.markup.MarkupModel
import com.intellij.openapi.editor.markup.RangeHighlighter
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiManager
import lang.ktav.lsp.UriUtil
import kotlin.math.min

/** Owns only this project's highlighters, including every split editor's real model. */
@Service(Service.Level.PROJECT)
class DiagnosticsRenderer(private val project: Project) : Disposable {
    private val log = Logger.getInstance(DiagnosticsRenderer::class.java)
    private data class OwnedHighlights(
        val uri: String,
        val model: MarkupModel,
        val highlighters: MutableList<RangeHighlighter>
    ) {
        fun clear() {
            for (highlighter in highlighters) {
                if (highlighter.isValid) model.removeHighlighter(highlighter)
            }
            highlighters.clear()
        }
    }

    // Confined to the EDT, as are editor creation/release and MarkupModel mutations.
    private val owned = mutableMapOf<Editor, OwnedHighlights>()
    @Volatile private var disposed = false

    init {
        EditorFactory.getInstance().addEditorFactoryListener(object : EditorFactoryListener {
            override fun editorCreated(event: EditorFactoryEvent) {
                val editor = event.editor
                if (editor.project !== project) return
                val file = FileDocumentManager.getInstance().getFile(editor.document) ?: return
                refresh(UriUtil.fromVirtualFile(file), editor.document)
            }

            override fun editorReleased(event: EditorFactoryEvent) {
                owned.remove(event.editor)?.clear()
            }
        }, this)
    }

    internal fun refresh(uri: String, previousDocument: Document? = null) {
        ApplicationManager.getApplication().invokeLater {
            if (disposed || project.isDisposed) return@invokeLater
            // Read at execution time, never capture diagnostics in a queued render.
            val publication = DiagnosticsHolder.getInstance(project).getPublication(uri)
            val document = publication?.snapshot?.document ?: previousDocument
            val iterator = owned.iterator()
            while (iterator.hasNext()) {
                val entry = iterator.next()
                if (entry.value.uri == uri || entry.key.isDisposed) {
                    entry.value.clear()
                    iterator.remove()
                }
            }
            publication?.withCurrent {
                for (editor in EditorFactory.getInstance().getEditors(publication.snapshot.document, project)) {
                    if (!editor.isDisposed) renderInEditor(uri, editor, publication.diagnostics)
                }
            }
            if (document != null) {
                val file = FileDocumentManager.getInstance().getFile(document)
                val psi = file?.let { PsiManager.getInstance(project).findFile(it) }
                if (psi != null) DaemonCodeAnalyzer.getInstance(project).restart(psi)
            }
        }
    }

    private fun renderInEditor(uri: String, editor: Editor, diagnostics: List<LspDiagnostic>) {
        if (diagnostics.isEmpty()) return
        val highlights = OwnedHighlights(uri, editor.markupModel, mutableListOf())
        owned[editor] = highlights
        val document = editor.document
        for (diag in diagnostics) {
            try {
                val start = posToOffset(document, diag.range.startLine, diag.range.startChar)
                val end = posToOffset(document, diag.range.endLine, diag.range.endChar)
                if (start < 0 || end < start) continue
                val finalEnd = if (start == end) document.getLineEndOffset(document.getLineNumber(start)) else end
                val key = when (diag.severity) {
                    1 -> CodeInsightColors.ERRORS_ATTRIBUTES
                    2 -> CodeInsightColors.WARNINGS_ATTRIBUTES
                    else -> CodeInsightColors.WEAK_WARNING_ATTRIBUTES
                }
                val highlighter = highlights.model.addRangeHighlighter(
                    start, finalEnd, HighlighterLayer.ERROR + 1,
                    editor.colorsScheme.getAttributes(key) ?: TextAttributes(),
                    HighlighterTargetArea.EXACT_RANGE
                )
                highlighter.errorStripeTooltip = diag.message
                highlights.highlighters.add(highlighter)
            } catch (ex: Exception) {
                log.warn("Failed to apply diagnostic: $diag", ex)
            }
        }
    }

    override fun dispose() {
        disposed = true
        val clear = Runnable {
            owned.values.forEach { it.clear() }
            owned.clear()
        }
        val application = ApplicationManager.getApplication()
        if (application.isDispatchThread) clear.run() else application.invokeLater(clear)
    }

    private fun posToOffset(document: Document, line: Int, char: Int): Int {
        if (line < 0 || line >= document.lineCount || char < 0) return -1
        val start = document.getLineStartOffset(line)
        return start + min(char, document.getLineEndOffset(line) - start)
    }

    companion object {
        fun getInstance(project: Project): DiagnosticsRenderer = project.getService(DiagnosticsRenderer::class.java)
    }
}
