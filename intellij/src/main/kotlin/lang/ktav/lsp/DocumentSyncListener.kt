package lang.ktav.lsp

import com.intellij.openapi.editor.Document
import com.intellij.openapi.editor.event.DocumentEvent
import com.intellij.openapi.editor.event.DocumentListener
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerListener
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.openapi.diagnostic.Logger

/**
 * Synchronizes document changes to LSP server.
 *
 * Listens to:
 * - File open: send didOpen
 * - File close: send didClose
 * - Document changes: send didChange
 */
class FileOpenListener(private val project: Project) : FileEditorManagerListener {

    private val log = Logger.getInstance(FileOpenListener::class.java)

    override fun fileOpened(source: FileEditorManager, file: VirtualFile) {
        if (file.extension != "ktav") return

        try {
            val lspService = project.getLspService()
            lspService.ensureInitialized()

            val client = lspService.getClient() ?: return
            val document = FileDocumentManager.getInstance().getDocument(file) ?: return
            val uri = file.url
            val text = document.text

            log.info("LSP: didOpen $uri")
            client.didOpen(uri, "ktav", 1, text)

            // Add change listener for this document
            document.addDocumentListener(FileChangeListener(project, file, document))
        } catch (ex: Exception) {
            log.error("Error on file open", ex)
        }
    }

    override fun fileClosed(source: FileEditorManager, file: VirtualFile) {
        if (file.extension != "ktav") return

        try {
            val lspService = project.getLspService()
            val client = lspService.getClient() ?: return
            val uri = file.url

            log.info("LSP: didClose $uri")
            client.didClose(uri)
        } catch (ex: Exception) {
            log.error("Error on file close", ex)
        }
    }
}

/**
 * Listens to document changes and sends didChange to LSP.
 */
class FileChangeListener(
    private val project: Project,
    private val file: VirtualFile,
    private val document: Document
) : DocumentListener {

    private val log = Logger.getInstance(FileChangeListener::class.java)
    private var version = 1

    override fun documentChanged(event: DocumentEvent) {
        try {
            val lspService = project.getLspService()
            val client = lspService.getClient() ?: return

            version++
            val uri = file.url
            val text = document.text

            log.debug("LSP: didChange $uri (version=$version)")
            client.didChange(uri, version, text)
        } catch (ex: Exception) {
            log.error("Error on document change", ex)
        }
    }
}
