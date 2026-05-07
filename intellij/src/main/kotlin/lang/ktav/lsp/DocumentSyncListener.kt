package lang.ktav.lsp

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Document
import com.intellij.openapi.editor.event.DocumentEvent
import com.intellij.openapi.editor.event.DocumentListener
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerListener
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import java.util.concurrent.ConcurrentHashMap

/**
 * Synchronizes document open/change/close events to LSP server.
 */
class FileOpenListener(private val project: Project) : FileEditorManagerListener {

    private val log = Logger.getInstance(FileOpenListener::class.java)

    init {
        log.info("[Ktav FileListener] Created for project: ${project.name}")
        println(">>> [Ktav FileListener] Created for project: ${project.name}")
    }

    override fun fileOpened(source: FileEditorManager, file: VirtualFile) {
        if (file.extension != "ktav") return

        log.info("[Ktav FileListener] .ktav file opened: ${file.url}")
        println(">>> [Ktav FileListener] Opened: ${file.url}")

        try {
            // Run LSP init in background to not block UI
            ApplicationManager.getApplication().executeOnPooledThread {
                try {
                    val lspService = project.getLspService()
                    log.info("[Ktav FileListener] Calling ensureInitialized()")
                    lspService.ensureInitialized()

                    val client = lspService.getClient()
                    if (client == null) {
                        log.warn("[Ktav FileListener] Client is null after ensureInitialized")
                        return@executeOnPooledThread
                    }

                    val document = ApplicationManager.getApplication().runReadAction<Document?> {
                        FileDocumentManager.getInstance().getDocument(file)
                    }
                    if (document == null) {
                        log.warn("[Ktav FileListener] Could not get document for $file")
                        return@executeOnPooledThread
                    }

                    val uri = file.url
                    val text = document.text
                    log.info("[Ktav FileListener] Sending didOpen for $uri (${text.length} chars)")
                    client.didOpen(uri, "ktav", 1, text)

                    // Add change listener for this document (only once per document)
                    val tracker = ChangeTracker.getInstance(project)
                    tracker.attachIfNeeded(file, document)
                } catch (ex: Exception) {
                    log.error("[Ktav FileListener] Error processing fileOpened", ex)
                }
            }
        } catch (ex: Exception) {
            log.error("[Ktav FileListener] Error scheduling fileOpened", ex)
        }
    }

    override fun fileClosed(source: FileEditorManager, file: VirtualFile) {
        if (file.extension != "ktav") return

        log.info("[Ktav FileListener] .ktav file closed: ${file.url}")
        println(">>> [Ktav FileListener] Closed: ${file.url}")

        try {
            val lspService = project.getLspService()
            val client = lspService.getClient()
            if (client == null) {
                log.warn("[Ktav FileListener] No client to send didClose")
                return
            }
            client.didClose(file.url)

            // Remove change listener
            ChangeTracker.getInstance(project).detach(file)
        } catch (ex: Exception) {
            log.error("[Ktav FileListener] Error on file close", ex)
        }
    }
}

/**
 * Tracks per-file document listeners to avoid registering duplicates.
 * Process-wide singleton (per-IDE).
 */
object ChangeTracker {
    private val log = Logger.getInstance(ChangeTracker::class.java)
    private val attached = ConcurrentHashMap<String, DocumentListener>()

    fun getInstance(project: Project): ChangeTracker = this

    fun attachIfNeeded(file: VirtualFile, document: Document) {
        val uri = file.url
        if (attached.containsKey(uri)) {
            log.debug("[Ktav ChangeTracker] Already attached: $uri")
            return
        }
        val listener = FileChangeListener(uri, document)
        document.addDocumentListener(listener)
        attached[uri] = listener
        log.info("[Ktav ChangeTracker] Attached change listener: $uri")
    }

    fun detach(file: VirtualFile) {
        val uri = file.url
        attached.remove(uri) ?: return
        log.info("[Ktav ChangeTracker] Detached change listener: $uri")
    }
}

/**
 * Listens to document changes and sends didChange to LSP.
 */
class FileChangeListener(
    private val uri: String,
    private val document: Document
) : DocumentListener {

    private val log = Logger.getInstance(FileChangeListener::class.java)
    @Volatile private var version = 1

    override fun documentChanged(event: DocumentEvent) {
        try {
            // Find LSP service via open projects (no project context here)
            val projects = com.intellij.openapi.project.ProjectManager.getInstance().openProjects
            for (project in projects) {
                val client = project.getLspService().getClient() ?: continue
                version++
                val text = document.text
                log.debug("[Ktav ChangeListener] didChange uri=$uri version=$version")
                client.didChange(uri, version, text)
                break
            }
        } catch (ex: Exception) {
            log.error("[Ktav ChangeListener] Error on documentChanged", ex)
        }
    }
}
