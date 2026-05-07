package lang.ktav.lsp

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.editor.colors.EditorColorsManager
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.HighlighterTargetArea
import com.intellij.openapi.editor.markup.MarkupModel
import com.intellij.openapi.editor.markup.RangeHighlighter
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.editor.colors.CodeInsightColors
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.TextEditor
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectManager
import com.intellij.openapi.vfs.VirtualFile
import java.util.concurrent.ConcurrentHashMap
import kotlin.math.min

/**
 * Applies LSP diagnostics directly to the editor's MarkupModel.
 *
 * Bypasses ExternalAnnotator/PSI requirements — works without a parser.
 */
object DiagnosticsRenderer {

    private val log = Logger.getInstance(DiagnosticsRenderer::class.java)

    // Track active highlighters per URI so we can clear them on update
    private val highlighters = ConcurrentHashMap<String, MutableList<RangeHighlighter>>()

    /**
     * Render diagnostics for a file. Called whenever publishDiagnostics arrives.
     */
    fun render(uri: String, diagnostics: List<LspDiagnostic>) {
        log.info("[Ktav Renderer] render: uri=$uri count=${diagnostics.size}")

        ApplicationManager.getApplication().invokeLater {
            try {
                applyToEditors(uri, diagnostics)
            } catch (ex: Exception) {
                log.error("[Ktav Renderer] Error rendering", ex)
            }
        }
    }

    private fun applyToEditors(uri: String, diagnostics: List<LspDiagnostic>) {
        for (project in ProjectManager.getInstance().openProjects) {
            if (project.isDisposed) continue

            val virtualFile = findVirtualFile(project, uri) ?: continue
            log.info("[Ktav Renderer] Found virtual file: $virtualFile")

            val editorManager = FileEditorManager.getInstance(project)
            val editors = editorManager.getEditors(virtualFile)

            for (fileEditor in editors) {
                if (fileEditor !is TextEditor) continue
                val editor = fileEditor.editor
                renderInEditor(uri, editor, diagnostics)
            }
        }
    }

    private fun findVirtualFile(project: Project, uri: String): VirtualFile? {
        // Match either file:/// or file:// form
        for (file in com.intellij.openapi.fileEditor.FileEditorManager.getInstance(project).openFiles) {
            if (UriUtil.fromVirtualFile(file) == uri) return file
        }
        return null
    }

    private fun renderInEditor(uri: String, editor: Editor, diagnostics: List<LspDiagnostic>) {
        val markupModel = editor.markupModel
        val document = editor.document

        // Clear previous highlighters for this URI
        val prev = highlighters.remove(uri) ?: emptyList()
        for (h in prev) {
            try {
                markupModel.removeHighlighter(h)
            } catch (_: Exception) {}
        }

        if (diagnostics.isEmpty()) return

        val newHighlighters = mutableListOf<RangeHighlighter>()
        val colorsScheme = EditorColorsManager.getInstance().globalScheme

        for (diag in diagnostics) {
            try {
                val startOffset = posToOffset(document, diag.range.startLine, diag.range.startChar)
                val endOffset = posToOffset(document, diag.range.endLine, diag.range.endChar)

                if (startOffset < 0 || endOffset < startOffset) {
                    log.warn("[Ktav Renderer] Bad range: $startOffset..$endOffset for $diag")
                    continue
                }
                val finalEnd = if (startOffset == endOffset) {
                    // Empty range — extend to end of line
                    val line = document.getLineNumber(startOffset)
                    if (line < document.lineCount) document.getLineEndOffset(line) else startOffset + 1
                } else endOffset

                val severityKey = when (diag.severity) {
                    1 -> CodeInsightColors.ERRORS_ATTRIBUTES
                    2 -> CodeInsightColors.WARNINGS_ATTRIBUTES
                    3 -> CodeInsightColors.INFO_ATTRIBUTES
                    4 -> CodeInsightColors.WEAK_WARNING_ATTRIBUTES
                    else -> CodeInsightColors.INFO_ATTRIBUTES
                }
                val attrs: TextAttributes = colorsScheme.getAttributes(severityKey)
                    ?: TextAttributes()

                val highlighter = markupModel.addRangeHighlighter(
                    startOffset,
                    finalEnd,
                    HighlighterLayer.ERROR + 1,
                    attrs,
                    HighlighterTargetArea.EXACT_RANGE
                )
                highlighter.errorStripeTooltip = diag.message
                newHighlighters.add(highlighter)

                log.info("[Ktav Renderer] Added highlighter [$startOffset..$finalEnd] severity=${diag.severity}: ${diag.message}")
            } catch (ex: Exception) {
                log.warn("[Ktav Renderer] Failed to apply diagnostic: $diag", ex)
            }
        }

        if (newHighlighters.isNotEmpty()) {
            highlighters[uri] = newHighlighters
        }
    }

    private fun posToOffset(document: com.intellij.openapi.editor.Document, line: Int, char: Int): Int {
        if (line < 0 || line >= document.lineCount) return -1
        val lineStart = document.getLineStartOffset(line)
        val lineEnd = document.getLineEndOffset(line)
        val charOffset = min(char, lineEnd - lineStart)
        return lineStart + charOffset
    }
}
