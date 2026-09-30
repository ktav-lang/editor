package lang.ktav.lsp.lifecycle

import lang.ktav.lsp.client.KtavServerDiscovery
import lang.ktav.lsp.diagnostics.DiagnosticsHolder
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import lang.ktav.lsp.client.KtavLspClient
import com.intellij.openapi.Disposable
import com.intellij.openapi.application.ApplicationManager
import com.google.gson.JsonElement
import lang.ktav.lsp.diagnostics.DiagnosticsRenderer
import java.lang.ref.WeakReference
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit

/**
 * Project-level service that manages LSP client lifecycle.
 *
 * Initialized once per project:
 * - Starts LSP server process (bundled ktav-lsp)
 * - Maintains connection
 * - Syncs documents
 * - Publishes diagnostics
 * - Cleans up on project close
 */
@Service(Service.Level.PROJECT)
class KtavLspProjectService(private val project: Project) : Disposable, AutoCloseable {

    private val log = Logger.getInstance(KtavLspProjectService::class.java)
    private val lifecycle = ProjectClientLifecycle(
        { project.isDisposed }, ::createClient, KtavLspClient::initialize, KtavLspClient::notifyInitialized
    ) { ex -> log.warn("[Ktav LSP] Client lifecycle failed", ex) }

    init {
        log.info("[Ktav LSP] KtavLspProjectService created for project: ${project.name}")
        println(">>> [Ktav LSP] KtavLspProjectService created for project: ${project.name}")
    }

    /**
     * Initialize LSP client (lazy initialization on first .ktav file).
     */
    fun ensureInitialized() = lifecycle.ensureInitialized()

    private fun createClient(): KtavLspClient {
        val serverCommand = KtavServerDiscovery.resolve().firstOrNull()
            ?: throw IllegalStateException("ktav-lsp binary not found in any location")
        val workspaceRoot = project.basePath
            ?: throw IllegalStateException("Project has no base path")
        val owner = WeakReference(project)
        lateinit var client: KtavLspClient
        client = KtavLspClient(
            serverCommand = serverCommand,
            workspaceRoot = workspaceRoot,
            onDiagnostics = { params ->
                val activeProject = owner.get()
                if (activeProject != null && !activeProject.isDisposed) {
                    // Read access precedes lifecycle/tracker locks, matching formatting.
                    ApplicationManager.getApplication().runReadAction {
                        if (!activeProject.isDisposed) {
                            val service = activeProject.getLspService()
                            service.withClient(client) { service.publishDiagnostics(client, params) }
                        }
                    }
                }
            }
        )
        return client
    }

    internal fun publishDiagnostics(clientIdentity: Any, params: JsonElement?): Boolean {
        if (project.isDisposed) return false
        return DiagnosticsHolder.getInstance(project).handlePublishDiagnostics(clientIdentity, params)
    }

    /**
     * Get initialized LSP client (or null if not initialized).
     */
    fun getClient(): KtavLspClient? = lifecycle.getClient()

    internal fun withClient(client: KtavLspClient, action: () -> Unit): Boolean = lifecycle.withClient(client, action)

    override fun dispose() = close()

    override fun close() {
        lifecycle.close()
        project.getServiceIfCreated(ChangeTracker::class.java)?.dispose()
        project.getServiceIfCreated(DiagnosticsHolder::class.java)?.dispose()
        project.getServiceIfCreated(DiagnosticsRenderer::class.java)?.dispose()
    }
}

internal class ProjectClientLifecycle<C : AutoCloseable>(
    private val isDisposed: () -> Boolean,
    private val create: () -> C,
    private val initialize: (C) -> CompletableFuture<Void>,
    private val notifyInitialized: (C) -> Unit,
    private val timeoutMillis: Long = 15_000,
    private val onFailure: (Exception) -> Unit
) : AutoCloseable {
    private val lifecycleLock = Any()
    @Volatile private var client: C? = null
    @Volatile private var closed = false
    private var failed = false
    private var session: Session? = null

    // Serializes callers, but close never waits for discovery or initialization.
    @Synchronized
    fun ensureInitialized() {
        synchronized(lifecycleLock) {
            if (closed || isDisposed() || failed || client != null) return
        }
        var candidate: Session? = null
        var accepted = false
        try {
            val owned = Session(create())
            candidate = owned
            val pending = synchronized(lifecycleLock) {
                if (closed || isDisposed()) return
                session = owned
                owned.start()
            }
            // Do not mutate the raw future with orTimeout: startup may outlive this wait.
            pending.get(timeoutMillis, TimeUnit.MILLISECONDS)
            synchronized(lifecycleLock) {
                if (closed || isDisposed() || session !== owned) return
                notifyInitialized(owned.value)
                if (closed || isDisposed() || session !== owned) return
                client = owned.value
                accepted = true
            }
        } catch (ex: Exception) {
            synchronized(lifecycleLock) { failed = true }
            if (ex is InterruptedException) Thread.currentThread().interrupt()
            if (!closed && !isDisposed()) onFailure(ex)
        } finally {
            if (!accepted) {
                synchronized(lifecycleLock) {
                    if (session === candidate) {
                        session = null
                        client = null
                    }
                }
                candidate?.close()
            }
        }
    }

    fun getClient(): C? = if (closed || isDisposed()) null else client

    fun withClient(identity: C, action: () -> Unit): Boolean = synchronized(lifecycleLock) {
        if (closed || isDisposed() || client !== identity) return@synchronized false
        action()
        true
    }

    override fun close() {
        val owned = synchronized(lifecycleLock) {
            closed = true
            client = null
            session.also { session = null }
        }
        owned?.close()
    }

    private inner class Session(val value: C) : AutoCloseable {
        private var pending: CompletableFuture<Void>? = null
        private var closeRequested = false
        private var finalCleanup = false

        fun start(): CompletableFuture<Void> {
            val future = initialize(value)
            pending = future
            future.whenComplete { _, _ ->
                synchronized(this) {
                    if (closeRequested && !finalCleanup) {
                        finalCleanup = true
                        cleanup()
                    }
                }
            }
            return future
        }

        @Synchronized
        override fun close() {
            if (closeRequested) return
            closeRequested = true
            finalCleanup = pending?.isDone != false
            cleanup()
        }

        private fun cleanup() {
            try {
                value.close()
            } catch (ex: Exception) {
                onFailure(ex)
            }
        }
    }
}

/**
 * Get the LSP service for a project.
 */
fun Project.getLspService(): KtavLspProjectService =
    getService(KtavLspProjectService::class.java)
