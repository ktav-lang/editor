package lang.ktav.lsp.lifecycle

import com.intellij.openapi.Disposable
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Document
import com.intellij.openapi.editor.event.DocumentEvent
import com.intellij.openapi.editor.event.DocumentListener
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerListener
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.VirtualFile
import lang.ktav.lsp.UriUtil
import lang.ktav.lsp.client.KtavLspClient
import lang.ktav.lsp.diagnostics.DiagnosticsHolder

class FileOpenListener(private val project: Project) : FileEditorManagerListener {
    private val log = Logger.getInstance(FileOpenListener::class.java)

    override fun fileOpened(source: FileEditorManager, file: VirtualFile) {
        if (source.project !== project || project.isDisposed || file.extension != "ktav") return

        val application = ApplicationManager.getApplication()
        application.executeOnPooledThread {
            try {
                if (project.isDisposed) return@executeOnPooledThread
                val service = project.getLspService()
                service.ensureInitialized()
                val client = service.getClient() ?: return@executeOnPooledThread

                // Open and attach atomically with respect to editor changes and close.
                application.invokeLater {
                    if (project.isDisposed || !file.isValid || !source.isFileOpen(file)) return@invokeLater
                    val document = FileDocumentManager.getInstance().getDocument(file) ?: return@invokeLater
                    service.withClient(client) {
                        ChangeTracker.getInstance(project).attachIfNeeded(file, document, client)
                    }
                }
            } catch (ex: Exception) {
                log.warn("[Ktav FileListener] Error processing fileOpened", ex)
            }
        }
    }

    override fun fileClosed(source: FileEditorManager, file: VirtualFile) {
        if (source.project !== project || project.isDisposed || file.extension != "ktav") return
        if (source.isFileOpen(file)) return
        ChangeTracker.getInstance(project).detach(file)
    }
}

internal class DocumentSyncConnection(
    val identity: Any,
    val didOpen: (String, Int, String) -> Unit,
    val didChange: (String, Int, String) -> Unit,
    val didClose: (String) -> Unit,
    val isActive: () -> Boolean = { true }
) {
    constructor(client: KtavLspClient, isActive: () -> Boolean) : this(
        client,
        { uri, version, text -> client.didOpen(uri, "ktav", version, text) },
        client::didChange,
        client::didClose,
        isActive
    )
}

/** One URI subscription per owner project; a shared Document can have several owners. */
@Service(Service.Level.PROJECT)
class ChangeTracker(private val project: Project) : Disposable {
    private val log = Logger.getInstance(ChangeTracker::class.java)
    private val attached = mutableMapOf<String, FileChangeListener>()
    private data class LastVersion(val client: Any, val version: Int)
    private val lastVersions = mutableMapOf<String, LastVersion>()
    private var disposed = false

    companion object {
        fun getInstance(project: Project): ChangeTracker = project.getService(ChangeTracker::class.java)
    }

    fun attachIfNeeded(file: VirtualFile, document: Document, client: KtavLspClient) {
        val service = project.getLspService()
        attach(UriUtil.fromVirtualFile(file), document, DocumentSyncConnection(client) {
            !project.isDisposed && service.getClient() === client
        })
    }

    @Synchronized
    internal fun attach(uri: String, document: Document, connection: DocumentSyncConnection) {
        if (disposed || project.isDisposed || !connection.isActive()) return
        val existing = attached[uri]
        if (existing?.document === document && existing.connection.identity === connection.identity) return
        detach(uri)

        val previous = lastVersions[uri]
        val version = if (previous?.client === connection.identity) previous.version + 1 else 1
        val listener = FileChangeListener(project, uri, document, connection, this, version)
        lastVersions[uri] = LastVersion(connection.identity, version)
        try {
            document.addDocumentListener(listener, listener)
            attached[uri] = listener
            listener.syncedStamp = document.modificationStamp
            connection.didOpen(uri, listener.version, document.text)
        } catch (ex: Exception) {
            attached.remove(uri)
            invalidateDiagnostics(uri)
            Disposer.dispose(listener)
            log.warn("[Ktav ChangeTracker] Could not open $uri", ex)
        }
    }

    fun detach(file: VirtualFile) = detach(UriUtil.fromVirtualFile(file))

    @Synchronized
    internal fun detach(uri: String) {
        val listener = attached.remove(uri) ?: return
        lastVersions[uri] = LastVersion(listener.connection.identity, listener.version)
        invalidateDiagnostics(uri)
        Disposer.dispose(listener)
        try {
            listener.connection.didClose(uri)
        } catch (ex: Exception) {
            log.warn("[Ktav ChangeTracker] Could not close $uri", ex)
        }
    }

    internal fun invalidateDiagnostics(uri: String) {
        if (project.isDisposed) return
        project.getServiceIfCreated(DiagnosticsHolder::class.java)?.clear(uri)
    }

    // Versions remain monotonic across close/reopen for the same client. A delayed
    // publish contains only URI/version, so resetting to 1 would alias sessions.
    @Synchronized
    internal fun diagnosticSnapshot(uri: String, clientIdentity: Any, version: Int): Snapshot? {
        if (disposed || project.isDisposed) return null
        val listener = attached[uri] ?: return null
        if (listener.version != version || listener.connection.identity !== clientIdentity || !listener.connection.isActive()) return null
        return captureSnapshot(uri, listener)
    }

    internal class Snapshot(val document: Document, private val check: (() -> Unit) -> Boolean) {
        fun isCurrent(): Boolean = check {}
        fun withCurrent(action: () -> Unit): Boolean = check(action)
    }

    // Call under a read action so the stamp, text and sent version agree.
    @Synchronized
    internal fun snapshot(uri: String, document: Document, clientIdentity: Any, text: String): Snapshot? {
        if (disposed || project.isDisposed) return null
        val listener = attached[uri] ?: return null
        if (listener.document !== document || listener.connection.identity !== clientIdentity || !listener.connection.isActive()) return null
        if (document.text != text) return null
        return captureSnapshot(uri, listener)
    }

    // Both callers hold the tracker lock and a read action.
    private fun captureSnapshot(uri: String, listener: FileChangeListener): Snapshot? {
        val document = listener.document
        val stamp = document.modificationStamp
        if (listener.syncedStamp != stamp) return null
        val version = listener.version
        return Snapshot(document) { action ->
            synchronized(this) {
                val current = !disposed && !project.isDisposed && attached[uri] === listener &&
                    listener.version == version && listener.syncedStamp == stamp &&
                    document.modificationStamp == stamp && listener.connection.isActive()
                if (current) action()
                current
            }
        }
    }

    @Synchronized
    override fun dispose() {
        if (disposed) return
        disposed = true
        attached.keys.toList().forEach { detach(it) }
        lastVersions.clear()
    }
}

internal class FileChangeListener(
    private val project: Project,
    private val uri: String,
    val document: Document,
    val connection: DocumentSyncConnection,
    private val owner: ChangeTracker,
    initialVersion: Int
) : DocumentListener, Disposable {
    private val log = Logger.getInstance(FileChangeListener::class.java)
    @Volatile var version = initialVersion
        private set
    @Volatile var syncedStamp: Long? = null
    @Volatile private var disposed = false

    override fun beforeDocumentChange(event: DocumentEvent) = synchronized(owner) {
        if (!disposed) owner.invalidateDiagnostics(uri)
    }

    override fun documentChanged(event: DocumentEvent) = synchronized(owner) {
        if (disposed || project.isDisposed) return@synchronized
        version++
        syncedStamp = null
        if (!connection.isActive()) return@synchronized
        try {
            connection.didChange(uri, version, document.text)
            syncedStamp = document.modificationStamp
        } catch (ex: Exception) {
            log.warn("[Ktav ChangeListener] Could not synchronize $uri", ex)
        }
    }

    override fun dispose() {
        disposed = true
        syncedStamp = null
    }
}
