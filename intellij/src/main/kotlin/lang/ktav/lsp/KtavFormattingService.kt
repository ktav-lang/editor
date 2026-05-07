package lang.ktav.lsp

import com.google.gson.JsonObject
import com.intellij.formatting.service.AsyncDocumentFormattingService
import com.intellij.formatting.service.AsyncFormattingRequest
import com.intellij.formatting.service.FormattingService
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.fileTypes.FileType
import com.intellij.psi.PsiFile
import lang.ktav.KtavFileType
import java.util.EnumSet

/**
 * Hooks the standard Reformat Code (Ctrl+Alt+L) action into LSP
 * `textDocument/formatting` for `.ktav` files.
 *
 * IntelliJ resolves `ReformatCode` via the registered FormattingService
 * extensions; ours wins for `.ktav` because we declare we handle the
 * full document of that file type. The actual work happens off-EDT in
 * [formatDocument]; the result is applied by the platform under a write
 * action.
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

    override fun createFormattingTask(request: AsyncFormattingRequest): FormattingTask? {
        val ctx = request.context
        val project = ctx.project
        val virtualFile = ctx.virtualFile ?: return null
        if (virtualFile.extension != "ktav") return null

        val client = project.getLspService().getClient()
        if (client == null) {
            log.warn("[Ktav FmtSvc] LSP client not initialized")
            return null
        }

        val uri = UriUtil.fromVirtualFile(virtualFile)
        val params = JsonObject().apply {
            add("textDocument", JsonObject().apply { addProperty("uri", uri) })
            add("options", JsonObject().apply {
                addProperty("tabSize", 4)
                addProperty("insertSpaces", true)
            })
        }

        return object : FormattingTask {
            @Volatile
            private var cancelled = false

            override fun run() {
                try {
                    log.info("[Ktav FmtSvc] Requesting formatting for $uri")
                    val result = client.sendFormatting(params)
                        .get(15, java.util.concurrent.TimeUnit.SECONDS)

                    if (cancelled) return
                    if (result == null || !result.isJsonArray) {
                        log.info("[Ktav FmtSvc] Server returned null/non-array → unchanged")
                        // Empty edits → no replacement. Pass current text back.
                        request.onTextReady(request.documentText)
                        return
                    }
                    val edits = result.asJsonArray
                    if (edits.size() == 0) {
                        log.info("[Ktav FmtSvc] No edits — already canonical")
                        request.onTextReady(request.documentText)
                        return
                    }

                    // Our LSP returns a single full-document replacement; if
                    // it ever sends multi-edit responses, we still apply
                    // them in reverse-offset order to a working copy.
                    val originalText = request.documentText
                    val finalText = applyEdits(originalText, edits)
                    log.info("[Ktav FmtSvc] Applying ${edits.size()} edit(s); newLen=${finalText.length}")
                    request.onTextReady(finalText)
                } catch (ex: Exception) {
                    log.warn("[Ktav FmtSvc] Formatting failed", ex)
                    request.onError("Ktav formatting failed", ex.message ?: "unknown error")
                }
            }

            override fun cancel(): Boolean {
                cancelled = true
                return true
            }

            override fun isRunUnderProgress(): Boolean = true
        }
    }

    private fun applyEdits(original: String, edits: com.google.gson.JsonArray): String {
        // Compute byte offsets via line counting on the original text.
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
