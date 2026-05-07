package lang.ktav.lsp

import com.google.gson.JsonElement
import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.ExternalAnnotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Document
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiFile
import java.util.concurrent.ConcurrentHashMap
import kotlin.math.min

/**
 * Diagnostics from LSP server, keyed by file URI.
 * Application-scoped service so any annotator can read latest diagnostics.
 */
@Service(Service.Level.APP)
class DiagnosticsHolder {

    private val log = Logger.getInstance(DiagnosticsHolder::class.java)
    private val diagnostics = ConcurrentHashMap<String, List<LspDiagnostic>>()

    fun setDiagnostics(fileUri: String, diags: List<LspDiagnostic>) {
        diagnostics[fileUri] = diags
        log.info("[Ktav Diagnostics] Stored ${diags.size} diagnostics for $fileUri")
    }

    fun getDiagnostics(fileUri: String): List<LspDiagnostic> =
        diagnostics[fileUri] ?: emptyList()

    fun clear(fileUri: String) {
        diagnostics.remove(fileUri)
    }

    companion object {
        private val companionLog = Logger.getInstance(DiagnosticsHolder::class.java)

        fun getInstance(): DiagnosticsHolder =
            ApplicationManager.getApplication().getService(DiagnosticsHolder::class.java)

        /**
         * Parse LSP publishDiagnostics params and store them.
         */
        fun handlePublishDiagnostics(params: JsonElement?) {
            companionLog.info("[Ktav Diagnostics] handlePublishDiagnostics called: $params")
            if (params == null || !params.isJsonObject) {
                companionLog.warn("[Ktav Diagnostics] params is null or not object")
                return
            }
            val obj = params.asJsonObject
            val uri = obj.get("uri")?.asString
            if (uri == null) {
                companionLog.warn("[Ktav Diagnostics] uri missing in params")
                return
            }
            val diagsArray = obj.getAsJsonArray("diagnostics")
            if (diagsArray == null) {
                companionLog.warn("[Ktav Diagnostics] diagnostics array missing")
                return
            }
            companionLog.info("[Ktav Diagnostics] uri=$uri, count=${diagsArray.size()}")

            val diagnostics = mutableListOf<LspDiagnostic>()
            for (elem in diagsArray) {
                try {
                    val d = elem.asJsonObject
                    val range = d.getAsJsonObject("range")
                    val start = range.getAsJsonObject("start")
                    val end = range.getAsJsonObject("end")

                    val diag = LspDiagnostic(
                        range = LspRange(
                            startLine = start.get("line").asInt,
                            startChar = start.get("character").asInt,
                            endLine = end.get("line").asInt,
                            endChar = end.get("character").asInt
                        ),
                        message = d.get("message")?.asString ?: "",
                        severity = d.get("severity")?.asInt,
                        code = d.get("code")?.asString
                    )
                    diagnostics.add(diag)
                    companionLog.info("[Ktav Diagnostics]   - ${diag.severity}/${diag.code}: ${diag.message} @${diag.range}")
                } catch (ex: Exception) {
                    companionLog.warn("[Ktav Diagnostics] Failed to parse diagnostic: $elem", ex)
                }
            }

            getInstance().setDiagnostics(uri, diagnostics)
        }
    }
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
 * ExternalAnnotator that reads diagnostics from holder (filled by LSP client)
 * and displays them as annotations in editor.
 */
class KtavDiagnosticsAnnotator : ExternalAnnotator<PsiFile, List<LspDiagnostic>>() {

    private val log = Logger.getInstance(KtavDiagnosticsAnnotator::class.java)

    override fun collectInformation(file: PsiFile): PsiFile {
        log.debug("[Ktav Annotator] collectInformation: ${file.virtualFile?.url}")
        return file
    }

    override fun doAnnotate(collectedInfo: PsiFile?): List<LspDiagnostic> {
        if (collectedInfo == null) return emptyList()
        val file = collectedInfo.virtualFile ?: return emptyList()
        val diags = DiagnosticsHolder.getInstance().getDiagnostics(file.url)
        log.info("[Ktav Annotator] doAnnotate: ${file.url} → ${diags.size} diagnostics")
        return diags
    }

    override fun apply(
        file: PsiFile,
        diagnostics: List<LspDiagnostic>,
        holder: AnnotationHolder
    ) {
        log.info("[Ktav Annotator] apply: ${file.virtualFile?.url} with ${diagnostics.size} diagnostics")
        val virtualFile = file.virtualFile ?: return
        val document = FileDocumentManager.getInstance().getDocument(virtualFile) ?: return

        for (diagnostic in diagnostics) {
            try {
                val range = diagnostic.range
                val startOffset = getOffset(document, range.startLine, range.startChar)
                val endOffset = getOffset(document, range.endLine, range.endChar)

                if (startOffset >= 0 && endOffset >= startOffset) {
                    val textRange = if (startOffset == endOffset) {
                        // Empty range — extend to end of line for visibility
                        val lineEnd = if (range.startLine < document.lineCount)
                            document.getLineEndOffset(range.startLine) else startOffset
                        TextRange(startOffset, lineEnd)
                    } else {
                        TextRange(startOffset, endOffset)
                    }

                    val severity = when (diagnostic.severity) {
                        1 -> HighlightSeverity.ERROR
                        2 -> HighlightSeverity.WARNING
                        3 -> HighlightSeverity.INFORMATION
                        4 -> HighlightSeverity.WEAK_WARNING
                        else -> HighlightSeverity.INFORMATION
                    }

                    holder.newAnnotation(severity, diagnostic.message)
                        .range(textRange)
                        .create()
                }
            } catch (ex: Exception) {
                log.warn("Failed to apply diagnostic: ${diagnostic.message}", ex)
            }
        }
    }

    private fun getOffset(document: Document, line: Int, char: Int): Int {
        if (line < 0 || line >= document.lineCount) return -1
        val lineStart = document.getLineStartOffset(line)
        val lineEnd = document.getLineEndOffset(line)
        val lineLength = lineEnd - lineStart
        val charOffset = min(char, lineLength)
        return lineStart + charOffset
    }
}
