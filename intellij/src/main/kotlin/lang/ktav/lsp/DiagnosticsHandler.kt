package lang.ktav.lsp

import com.google.gson.JsonObject
import com.intellij.lang.annotation.AnnotationBuilder
import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.ExternalAnnotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Document
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiFile
import kotlin.math.min

/**
 * Receives textDocument/publishDiagnostics from LSP server
 * and displays them in the editor.
 *
 * Diagnostics are LSP objects with:
 * - range: { start: { line, character }, end: { line, character } }
 * - message: string
 * - severity: 1 (error), 2 (warning), 3 (info), 4 (hint)
 * - code: optional
 */
class DiagnosticsHolder {
    // diagnostics per file URI
    private val diagnostics = mutableMapOf<String, List<LspDiagnostic>>()

    fun setDiagnostics(fileUri: String, diags: List<LspDiagnostic>) {
        diagnostics[fileUri] = diags
    }

    fun getDiagnostics(fileUri: String): List<LspDiagnostic> =
        diagnostics[fileUri] ?: emptyList()
}

data class LspDiagnostic(
    val range: LspRange,
    val message: String,
    val severity: Int?, // 1=error, 2=warning, 3=info, 4=hint
    val code: String?
)

data class LspRange(
    val startLine: Int,
    val startChar: Int,
    val endLine: Int,
    val endChar: Int
)

/**
 * ExternalAnnotator that reads diagnostics from holder
 * and displays them as annotations in editor.
 */
class KtavDiagnosticsAnnotator : ExternalAnnotator<PsiFile, List<LspDiagnostic>>() {

    private val log = Logger.getInstance(KtavDiagnosticsAnnotator::class.java)
    private val diagnosticsHolder = DiagnosticsHolder()

    fun setDiagnostics(fileUri: String, diags: List<LspDiagnostic>) {
        diagnosticsHolder.setDiagnostics(fileUri, diags)
    }

    override fun collectInformation(file: PsiFile): PsiFile? {
        // We collect diagnostics asynchronously via LSP, not by examining the PSI tree
        return file
    }

    override fun doAnnotate(collectedInfo: PsiFile?): List<LspDiagnostic> {
        if (collectedInfo == null) return emptyList()

        val file = collectedInfo.virtualFile ?: return emptyList()
        val uri = file.url
        return diagnosticsHolder.getDiagnostics(uri)
    }

    override fun apply(
        file: PsiFile,
        diagnostics: List<LspDiagnostic>,
        holder: AnnotationHolder
    ) {
        val document = FileDocumentManager.getInstance().getDocument(file.virtualFile ?: return)
            ?: return

        for (diagnostic in diagnostics) {
            try {
                val range = diagnostic.range

                // Convert LSP range (line, char) to IntelliJ offset
                val startOffset = getOffset(document, range.startLine, range.startChar)
                val endOffset = getOffset(document, range.endLine, range.endChar)

                if (startOffset >= 0 && endOffset >= startOffset) {
                    val textRange = TextRange(startOffset, endOffset)

                    val severity = when (diagnostic.severity) {
                        1 -> HighlightSeverity.ERROR
                        2 -> HighlightSeverity.WARNING
                        3 -> HighlightSeverity.INFORMATION
                        4 -> HighlightSeverity.WEAK_WARNING
                        else -> HighlightSeverity.INFORMATION
                    }

                    val annotation = holder.newAnnotation(severity, diagnostic.message)
                        .range(textRange)

                    if (diagnostic.code != null) {
                        annotation.problemGroup { "Ktav: ${diagnostic.code}" }
                    }

                    annotation.create()
                }
            } catch (ex: Exception) {
                log.warn("Failed to apply diagnostic: ${diagnostic.message}", ex)
            }
        }
    }

    /**
     * Convert (line, char) to document offset.
     */
    private fun getOffset(document: Document, line: Int, char: Int): Int {
        if (line < 0 || line >= document.lineCount) return -1

        val lineStart = document.getLineStartOffset(line)
        val lineEnd = document.getLineEndOffset(line)
        val lineLength = lineEnd - lineStart
        val charOffset = min(char, lineLength)

        return lineStart + charOffset
    }
}
