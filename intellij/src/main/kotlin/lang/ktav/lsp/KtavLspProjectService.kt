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
    private var initFailed = false

    init {
        log.info("[Ktav LSP] KtavLspProjectService created for project: ${project.name}")
        println(">>> [Ktav LSP] KtavLspProjectService created for project: ${project.name}")
    }

    /**
     * Initialize LSP client (lazy initialization on first .ktav file).
     */
    @Synchronized
    fun ensureInitialized() {
        if (client != null) {
            log.info("[Ktav LSP] Already initialized, reusing existing client")
            return
        }
        if (isInitializing) {
            log.info("[Ktav LSP] Already initializing, waiting...")
            return
        }
        if (initFailed) {
            log.warn("[Ktav LSP] Previous init failed, not retrying")
            return
        }

        isInitializing = true
        try {
            log.info("[Ktav LSP] Starting initialization sequence")
            println(">>> [Ktav LSP] Starting initialization sequence")

            // Resolve server command
            val command = KtavServerDiscovery.resolve()
            log.info("[Ktav LSP] Discovery returned command: $command")
            println(">>> [Ktav LSP] Discovery returned command: $command")

            val serverCommand = command.firstOrNull()
                ?: throw IllegalStateException("ktav-lsp binary not found in any location")

            // Verify it's executable
            log.info("[Ktav LSP] Server command: '$serverCommand'")
            val cmdFile = java.io.File(serverCommand)
            if (cmdFile.exists()) {
                log.info("[Ktav LSP] Binary exists: ${cmdFile.absolutePath} (${cmdFile.length()} bytes, executable=${cmdFile.canExecute()})")
            } else {
                log.warn("[Ktav LSP] Binary path doesn't exist as file (may be on PATH): $serverCommand")
            }

            // Resolve workspace root
            val workspaceRoot = project.basePath
                ?: throw IllegalStateException("Project has no base path")
            log.info("[Ktav LSP] Workspace root: $workspaceRoot")

            // Create client
            client = KtavLspClient(
                serverCommand = serverCommand,
                workspaceRoot = workspaceRoot,
                onDiagnostics = { params ->
                    log.info("[Ktav LSP] Received diagnostics callback")
                    DiagnosticsHolder.handlePublishDiagnostics(params)
                },
                onPostDiagnostics = {
                    log.info("[Ktav LSP] Triggering DaemonCodeAnalyzer restart")
                    com.intellij.openapi.application.ApplicationManager.getApplication().invokeLater {
                        if (!project.isDisposed) {
                            com.intellij.codeInsight.daemon.DaemonCodeAnalyzer.getInstance(project).restart()
                        }
                    }
                }
            )

            log.info("[Ktav LSP] Client created, calling initialize()")
            client!!.initialize().thenRun {
                log.info("[Ktav LSP] Initialize completed, sending initialized notification")
                client!!.notifyInitialized()
                log.info("[Ktav LSP] Initialized notification sent. LSP fully active.")
                println(">>> [Ktav LSP] LSP fully active")
            }.exceptionally { ex ->
                log.error("[Ktav LSP] Failed to initialize LSP client", ex)
                println(">>> [Ktav LSP] Init failed: ${ex.message}")
                initFailed = true
                client?.close()
                client = null
                null
            }.get()
        } catch (ex: Exception) {
            log.error("[Ktav LSP] Error ensuring LSP initialization", ex)
            println(">>> [Ktav LSP] Init error: ${ex.message}")
            initFailed = true
            client?.close()
            client = null
        } finally {
            isInitializing = false
        }
    }

    /**
     * Get initialized LSP client (or null if not initialized).
     */
    fun getClient(): KtavLspClient? = client

    override fun close() {
        log.info("[Ktav LSP] Closing project service")
        try {
            client?.close()
            client = null
            log.info("[Ktav LSP] Project service closed")
        } catch (ex: Exception) {
            log.error("[Ktav LSP] Error closing project service", ex)
        }
    }
}

/**
 * Get the LSP service for a project.
 */
fun Project.getLspService(): KtavLspProjectService =
    getService(KtavLspProjectService::class.java)
