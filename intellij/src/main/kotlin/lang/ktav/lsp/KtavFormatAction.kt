package lang.ktav.lsp

import com.google.gson.JsonObject
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Document
import com.intellij.openapi.editor.Editor
import lang.ktav.KtavFileType
import kotlin.math.min

/**
 * Reformat the current `.ktav` file via LSP `textDocument/formatting`.
 *
 * Workflow:
 *   1. Get current editor + document.
 *   2. Send `textDocument/formatting` request to ktav-lsp.
 *   3. Receive `TextEdit[]`; apply them to the document under a write action.
 *   4. Editor sees the updated text and re-paints.
 *
 * Bound to `Ctrl+Alt+L` (the standard reformat shortcut) for `.ktav` files.
 */
class KtavFormatAction : AnAction() {

    private val log = Logger.getInstance(KtavFormatAction::class.java)

    override fun getActionUpdateThread() = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        val file = e.getData(CommonDataKeys.PSI_FILE)
        e.presentation.isEnabledAndVisible = file?.fileType === KtavFileType
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val editor = e.getData(CommonDataKeys.EDITOR) ?: return
        val virtualFile = e.getData(CommonDataKeys.VIRTUAL_FILE) ?: return
        if (virtualFile.extension != "ktav") return

        val client = project.getLspService().getClient() ?: run {
            log.warn("[Ktav Format] LSP client not initialized — cannot format")
            return
        }

        val uri = UriUtil.fromVirtualFile(virtualFile)
        val params = JsonObject().apply {
            add("textDocument", JsonObject().apply { addProperty("uri", uri) })
            add("options", JsonObject().apply {
                addProperty("tabSize", editor.settings.getTabSize(project))
                addProperty("insertSpaces", true)
            })
        }

        log.info("[Ktav Format] Requesting formatting for $uri")
        ApplicationManager.getApplication().executeOnPooledThread {
            try {
                val response = client.sendFormatting(params).get(10, java.util.concurrent.TimeUnit.SECONDS)
                if (response == null || !response.isJsonArray) {
                    log.info("[Ktav Format] Server returned null/non-array — nothing to format")
                    return@executeOnPooledThread
                }
                val edits = response.asJsonArray
                if (edits.size() == 0) {
                    log.info("[Ktav Format] No edits — file already canonical")
                    return@executeOnPooledThread
                }
                log.info("[Ktav Format] Applying ${edits.size()} edit(s)")
                ApplicationManager.getApplication().invokeLater {
                    applyEdits(project, editor.document, edits)
                }
            } catch (ex: Exception) {
                log.warn("[Ktav Format] Formatting request failed", ex)
            }
        }
    }

    private fun applyEdits(
        project: com.intellij.openapi.project.Project,
        document: Document,
        edits: com.google.gson.JsonArray
    ) {
        WriteCommandAction.runWriteCommandAction(project, "Ktav: Format", null, {
            // Apply edits in reverse offset order so earlier offsets stay valid.
            val parsed = edits.map { it.asJsonObject }.map { obj ->
                val r = obj.getAsJsonObject("range")
                val s = r.getAsJsonObject("start")
                val e = r.getAsJsonObject("end")
                Triple(
                    posToOffset(document, s.get("line").asInt, s.get("character").asInt),
                    posToOffset(document, e.get("line").asInt, e.get("character").asInt),
                    obj.get("newText").asString
                )
            }.sortedByDescending { it.first }

            for ((startOff, endOff, newText) in parsed) {
                if (startOff < 0 || endOff < startOff) continue
                document.replaceString(startOff, endOff, newText)
            }
        })
    }

    private fun posToOffset(document: Document, line: Int, char: Int): Int {
        if (line < 0) return 0
        if (line >= document.lineCount) return document.textLength
        val lineStart = document.getLineStartOffset(line)
        val lineEnd = document.getLineEndOffset(line)
        val charOffset = min(char, lineEnd - lineStart)
        return lineStart + charOffset
    }
}
