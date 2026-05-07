package lang.ktav.lsp

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.ProjectActivity

/**
 * Runs after a project is opened.
 *
 * Initializes LSP client and sends didOpen for all already-opened .ktav files.
 * This is critical because FileEditorManagerListener only fires on NEW file
 * opens — files opened before the listener registration are missed.
 */
class KtavStartupActivity : ProjectActivity {

    private val log = Logger.getInstance(KtavStartupActivity::class.java)

    override suspend fun execute(project: Project) {
        log.info("[Ktav Startup] Project opened: ${project.name}, basePath=${project.basePath}")
        println(">>> [Ktav Startup] Project opened: ${project.name}")

        // Find any already-open .ktav files
        val editorManager = FileEditorManager.getInstance(project)
        val openFiles = editorManager.openFiles.filter { it.extension == "ktav" }
        log.info("[Ktav Startup] Found ${openFiles.size} already-open .ktav file(s)")

        if (openFiles.isEmpty()) {
            log.info("[Ktav Startup] No .ktav files open yet, LSP will start lazily on first open")
            return
        }

        // Start LSP and send didOpen for each
        ApplicationManager.getApplication().executeOnPooledThread {
            try {
                log.info("[Ktav Startup] Initializing LSP for ${openFiles.size} pre-open file(s)")
                val lspService = project.getLspService()
                lspService.ensureInitialized()

                val client = lspService.getClient()
                if (client == null) {
                    log.warn("[Ktav Startup] LSP client is null after init, aborting")
                    return@executeOnPooledThread
                }

                for (file in openFiles) {
                    val document = ApplicationManager.getApplication().runReadAction<com.intellij.openapi.editor.Document?> {
                        FileDocumentManager.getInstance().getDocument(file)
                    } ?: continue

                    log.info("[Ktav Startup] Sending didOpen for ${UriUtil.fromVirtualFile(file)}")
                    client.didOpen(UriUtil.fromVirtualFile(file), "ktav", 1, document.text)

                    ChangeTracker.attachIfNeeded(file, document)
                }
                log.info("[Ktav Startup] All pre-open files synced to LSP")
            } catch (ex: Exception) {
                log.error("[Ktav Startup] Failed to initialize LSP for pre-open files", ex)
            }
        }
    }
}
