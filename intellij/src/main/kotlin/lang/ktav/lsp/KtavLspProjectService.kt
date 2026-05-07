package lang.ktav.lsp

import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import lang.ktav.lsp.client.KtavLspClient

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
class KtavLspProjectService(private val project: Project) : AutoCloseable {

    private val log = Logger.getInstance(KtavLspProjectService::class.java)
    private var client: KtavLspClient? = null
    private var isInitializing = false

    /**
     * Initialize LSP client (lazy initialization on first .ktav file).
     */
    fun ensureInitialized() {
        if (client != null || isInitializing) return

        isInitializing = true
        try {
            val serverCommand = KtavServerDiscovery.resolve().firstOrNull()
                ?: throw IllegalStateException("ktav-lsp binary not found")

            val workspaceRoot = project.basePath
                ?: throw IllegalStateException("Project has no base path")

            log.info("Starting LSP client for project: $workspaceRoot")
            log.info("Server command: $serverCommand")

            client = KtavLspClient(serverCommand, workspaceRoot)
            client!!.initialize().thenRun {
                client!!.notifyInitialized()
                log.info("LSP client initialized")
            }.exceptionally { ex ->
                log.error("Failed to initialize LSP client", ex)
                null
            }.get()
        } catch (ex: Exception) {
            log.error("Error ensuring LSP initialization", ex)
            isInitializing = false
        }
    }

    /**
     * Get initialized LSP client (or null if not initialized).
     */
    fun getClient(): KtavLspClient? = client

    override fun close() {
        try {
            client?.close()
            client = null
            log.info("LSP project service closed")
        } catch (ex: Exception) {
            log.error("Error closing LSP project service", ex)
        }
    }
}

/**
 * Get the LSP service for a project.
 */
fun Project.getLspService(): KtavLspProjectService =
    getService(KtavLspProjectService::class.java)
