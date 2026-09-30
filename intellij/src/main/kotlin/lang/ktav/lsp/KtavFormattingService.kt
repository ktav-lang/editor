package lang.ktav.lsp

import lang.ktav.lsp.lifecycle.getLspService
import lang.ktav.lsp.lifecycle.ChangeTracker
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.intellij.formatting.service.AsyncDocumentFormattingService
import com.intellij.formatting.service.AsyncFormattingRequest
import com.intellij.formatting.service.FormattingService
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.application.ModalityState
import com.intellij.openapi.command.CommandProcessor
import com.intellij.openapi.editor.Document
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileTypes.FileType
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.psi.PsiFile
import lang.ktav.KtavFileType
import java.util.EnumSet
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit
import java.util.concurrent.TimeoutException

/**
 * Hooks the standard Reformat Code (Ctrl+Alt+L) action into LSP
 * `textDocument/formatting` for `.ktav` files.
 *
 * IntelliJ resolves `ReformatCode` via the registered FormattingService
 * extensions; ours wins for `.ktav` because we declare we handle the
 * full document of that file type. The actual work happens off-EDT in
 * [formatDocument]; the result is checked and applied in one write action.
 */
class KtavFormattingService : AsyncDocumentFormattingService() {

    private val log = Logger.getInstance(KtavFormattingService::class.java)

    override fun getFeatures(): MutableSet<FormattingService.Feature> {
        // Whole-file formatting only — Ktav has no meaningful sub-range
        // formatting (the indented-stripped form depends on outer indent).
        return EnumSet.noneOf(FormattingService.Feature::class.java)
    }

    override fun canFormat(file: PsiFile): Boolean {
        val ft: FileType = file.fileType
        return ft === KtavFileType
    }

    override fun getName(): String = "Ktav LSP Formatter"

    override fun getNotificationGroupId(): String = "Ktav"

    // The platform otherwise queues another write after onTextReady, losing our guard.
    override fun needToUpdate(): Boolean = false

    override fun createFormattingTask(request: AsyncFormattingRequest): FormattingTask? {
        val ctx = request.context
        val project = ctx.project
        val virtualFile = ctx.virtualFile ?: return null
        if (virtualFile.extension != "ktav") return null

        if (project.isDisposed) return null
        val service = project.getLspService()
        val client = service.getClient()
        if (client == null) {
            log.warn("[Ktav FmtSvc] LSP client not initialized")
            return null
        }

        val uri = UriUtil.fromVirtualFile(virtualFile)
        val application = ApplicationManager.getApplication()
        val modality = ModalityState.current()
        val document = application.runReadAction<Document?> {
            FileDocumentManager.getInstance().getCachedDocument(virtualFile)
        } ?: return null
        val originalText = application.runReadAction<String> { request.documentText }
        val snapshot = application.runReadAction<ChangeTracker.Snapshot?> {
            ChangeTracker.getInstance(project).snapshot(uri, document, client, originalText)
        } ?: return null
        val params = JsonObject().apply {
            add("textDocument", JsonObject().apply { addProperty("uri", uri) })
            add("options", JsonObject().apply {
                addProperty("tabSize", 4)
                addProperty("insertSpaces", true)
            })
        }

        val task = guardedTask(request, originalText, { client.sendFormatting(params) },
            dispatchResult = { action ->
                val apply = Runnable {
                    CommandProcessor.getInstance().executeCommand(project, {
                        application.runWriteAction { action() }
                    }, "Reformat Code", null)
                }
                if (application.isDispatchThread) apply.run() else application.invokeLater(apply, modality)
            },
            onTextReady = { text ->
                if (document.text != text) document.setText(text)
                request.onTextReady(text)
            }
        ) { action ->
            application.runReadAction<Boolean> {
                if (!virtualFile.isValid ||
                    FileDocumentManager.getInstance().getCachedDocument(virtualFile) !== document) {
                    false
                } else {
                    var current = false
                    service.withClient(client) { current = snapshot.withCurrent(action) }
                    current
                }
            }
        }
        return object : FormattingTask {
            override fun run() = task.run()
            override fun cancel(): Boolean = task.cancel()
            override fun isRunUnderProgress(): Boolean = true
        }
    }

    internal abstract class GuardedTask : Runnable {
        abstract fun cancel(): Boolean
    }

    internal fun guardedTask(
        request: AsyncFormattingRequest,
        originalText: String,
        sendFormatting: () -> CompletableFuture<JsonElement>,
        dispatchResult: (() -> Unit) -> Unit = { it() },
        onTextReady: (String) -> Unit = { request.onTextReady(it) },
        nanoTime: () -> Long = System::nanoTime,
        withCurrentSnapshot: (() -> Unit) -> Boolean
    ): GuardedTask {
        return object : GuardedTask() {
            private val deliveryLock = Any()
            @Volatile
            private var cancelled = false
            @Volatile
            private var responseFuture: CompletableFuture<JsonElement>? = null
            @Volatile
            private var progressIndicator: ProgressIndicator? = null

            private fun isCancelled(): Boolean = cancelled || progressIndicator?.isCanceled == true

            override fun run() {
                progressIndicator = ProgressManager.getInstance().progressIndicator
                try {
                    if (isCancelled()) return
                    val startedAt = nanoTime()
                    var response: CompletableFuture<JsonElement>? = null
                    if (!withCurrentSnapshot {
                        synchronized(deliveryLock) {
                            if (!isCancelled()) {
                                response = sendFormatting()
                                responseFuture = response
                            }
                        }
                    }) {
                        stale()
                        return
                    }
                    val pending = response ?: return
                    val result = pending.get(15, TimeUnit.SECONDS)
                    if (isCancelled()) return

                    val finalText = if (result != null && result.isJsonArray) applyEdits(originalText, result.asJsonArray) else originalText
                    // Recheck at delivery, not just when the response arrives.
                    dispatchResult {
                        try {
                            if (!isCancelled() && !withCurrentSnapshot {
                                synchronized(deliveryLock) {
                                    if (!isCancelled()) {
                                        if (nanoTime() - startedAt >= TimeUnit.SECONDS.toNanos(15)) {
                                            throw TimeoutException("Formatting response expired; retry formatting")
                                        }
                                        onTextReady(finalText)
                                    }
                                }
                            }) stale()
                        } catch (ex: Exception) {
                            failed(ex)
                        }
                    }
                } catch (ex: Exception) {
                    failed(ex)
                }
            }

            private fun stale() = synchronized(deliveryLock) {
                if (!isCancelled()) request.onError("Ktav formatting cancelled", "Document changed or was closed; retry formatting")
            }

            private fun failed(ex: Exception) {
                responseFuture?.cancel(false)
                if (ex is InterruptedException) Thread.currentThread().interrupt()
                synchronized(deliveryLock) {
                    if (isCancelled()) return
                    log.warn("[Ktav FmtSvc] Formatting failed", ex)
                    request.onError("Ktav formatting failed", ex.message ?: "unknown error")
                }
            }

            override fun cancel(): Boolean {
                synchronized(deliveryLock) { cancelled = true }
                responseFuture?.cancel(false)
                return true
            }

        }
    }

    private fun applyEdits(original: String, edits: com.google.gson.JsonArray): String {
        // LSP positions and Kotlin string offsets use UTF-16 code units.
        val sb = StringBuilder(original)
        // Sort edits by start offset descending so earlier edits stay valid.
        val parsed = edits.map { it.asJsonObject }.map { obj ->
            val r = obj.getAsJsonObject("range")
            val s = r.getAsJsonObject("start")
            val e = r.getAsJsonObject("end")
            Triple(
                posToOffset(original, s.get("line").asInt, s.get("character").asInt),
                posToOffset(original, e.get("line").asInt, e.get("character").asInt),
                obj.get("newText").asString
            )
        }.sortedByDescending { it.first }

        for ((startOff, endOff, newText) in parsed) {
            if (startOff < 0 || endOff < startOff) continue
            sb.replace(startOff, endOff, newText)
        }
        return sb.toString()
    }

    private fun posToOffset(text: String, line: Int, char: Int): Int {
        if (line < 0) return 0
        // Walk lines counting offsets.
        var lineIdx = 0
        var i = 0
        while (i < text.length && lineIdx < line) {
            if (text[i] == '\n') lineIdx++
            i++
        }
        if (lineIdx < line) return text.length
        // i is at start of target line; advance char chars (clipped to line length).
        var col = 0
        while (i < text.length && text[i] != '\n' && col < char) {
            i++
            col++
        }
        return i
    }
}
