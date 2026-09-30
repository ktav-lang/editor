package lang.ktav.lsp.diagnostics

import lang.ktav.lsp.UriUtil
import com.google.gson.JsonElement
import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.ExternalAnnotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.Disposable
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Document
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.project.Project
import lang.ktav.lsp.lifecycle.ChangeTracker
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiFile
import java.util.concurrent.ConcurrentHashMap
import kotlin.math.min

/** A publication is valid only while its exact synchronized document session is current. */
class DiagnosticPublication internal constructor(
    internal val project: Project,
    private val uri: String,
    internal val snapshot: ChangeTracker.Snapshot,
    val diagnostics: List<LspDiagnostic>
) {
    internal fun withCurrent(action: () -> Unit): Boolean {
        var applied = false
        snapshot.withCurrent {
            if (DiagnosticsHolder.getInstance(project).isPublished(uri, this)) {
                action()
                applied = true
            }
        }
        return applied
    }
}

@Service(Service.Level.PROJECT)
class DiagnosticsHolder(private val project: Project) : Disposable {
    private val log = Logger.getInstance(DiagnosticsHolder::class.java)
    private val publications = ConcurrentHashMap<String, DiagnosticPublication>()
    @Volatile private var disposed = false

    internal fun isPublished(uri: String, publication: DiagnosticPublication): Boolean =
        !disposed && publications[uri] === publication

    internal fun getPublication(uri: String): DiagnosticPublication? {
        if (disposed || project.isDisposed) return null
        return publications[uri]?.takeIf { it.snapshot.isCurrent() }
    }

    fun getDiagnostics(uri: String): List<LspDiagnostic> = getPublication(uri)?.diagnostics ?: emptyList()

    internal fun clear(uri: String) {
        val previous = publications.remove(uri)
        if (!disposed && !project.isDisposed) DiagnosticsRenderer.getInstance(project).refresh(uri, previous?.snapshot?.document)
    }

    internal fun handlePublishDiagnostics(clientIdentity: Any, params: JsonElement?): Boolean {
        ApplicationManager.getApplication().assertReadAccessAllowed()
        if (disposed || project.isDisposed || params == null || !params.isJsonObject) return false
        val obj = params.asJsonObject
        val uri = obj.get("uri")?.takeUnless { it.isJsonNull }?.asString?.let(UriUtil::normalize) ?: return false
        // The bundled server versions every open-document publication. Unversioned
        // didClose clears cannot be distinguished from delayed clears after reopen.
        val version = obj.get("version")?.takeUnless { it.isJsonNull }?.asInt ?: return false
        val array = obj.getAsJsonArray("diagnostics") ?: return false
        val snapshot = ChangeTracker.getInstance(project).diagnosticSnapshot(uri, clientIdentity, version)
            ?: return false
        val diagnostics = array.mapNotNull { elem ->
            try {
                val d = elem.asJsonObject
                val range = d.getAsJsonObject("range")
                val start = range.getAsJsonObject("start")
                val end = range.getAsJsonObject("end")
                LspDiagnostic(
                    LspRange(start.get("line").asInt, start.get("character").asInt,
                        end.get("line").asInt, end.get("character").asInt),
                    d.get("message")?.asString ?: "",
                    d.get("severity")?.asInt,
                    d.get("code")?.asString
                )
            } catch (ex: Exception) {
                log.warn("Failed to parse diagnostic: $elem", ex)
                null
            }
        }
        val accepted = snapshot.withCurrent {
            synchronized(this) {
                if (!disposed) publications[uri] = DiagnosticPublication(project, uri, snapshot, diagnostics)
            }
        }
        if (accepted && !disposed) DiagnosticsRenderer.getInstance(project).refresh(uri, snapshot.document)
        return accepted && !disposed
    }

    @Synchronized
    override fun dispose() {
        disposed = true
        publications.clear()
    }

    companion object {
        fun getInstance(project: Project): DiagnosticsHolder = project.getService(DiagnosticsHolder::class.java)
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
class KtavDiagnosticsAnnotator : ExternalAnnotator<PsiFile, DiagnosticPublication>() {

    private val log = Logger.getInstance(KtavDiagnosticsAnnotator::class.java)

    override fun collectInformation(file: PsiFile): PsiFile {
        log.debug("[Ktav Annotator] collectInformation: ${file.virtualFile?.url}")
        return file
    }

    override fun doAnnotate(collectedInfo: PsiFile?): DiagnosticPublication? {
        if (collectedInfo == null || collectedInfo.project.isDisposed) return null
        val file = collectedInfo.virtualFile ?: return null
        val uri = UriUtil.fromVirtualFile(file)
        return DiagnosticsHolder.getInstance(collectedInfo.project).getPublication(uri)
    }

    override fun apply(
        file: PsiFile,
        publication: DiagnosticPublication,
        holder: AnnotationHolder
    ) {
        if (file.project.isDisposed) return
        val virtualFile = file.virtualFile ?: return
        val document = FileDocumentManager.getInstance().getDocument(virtualFile) ?: return
        if (publication.project !== file.project || publication.snapshot.document !== document) return

        publication.withCurrent {
            for (diagnostic in publication.diagnostics) {
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
