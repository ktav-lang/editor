package lang.ktav.lsp.client

import com.google.gson.JsonObject
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.util.SystemInfo
import java.io.File
import java.util.concurrent.CompletableFuture

/**
 * Ktav LSP client: manages lifecycle and protocol communication.
 *
 * Lifecycle:
 * 1. Initialize: send initialize request, wait for initialized notification
 * 2. Sync documents: didOpen/didChange/didClose
 * 3. Shutdown: send shutdown request, exit
 */
class KtavLspClient(
    private val serverCommand: String,
    private val workspaceRoot: String
) : AutoCloseable {

    private val log = Logger.getInstance(KtavLspClient::class.java)
    private var transport: LspTransport? = null
    private var isInitialized = false

    /**
     * Start the LSP server process and send initialize request.
     */
    fun initialize(): CompletableFuture<Void> {
        return CompletableFuture.runAsync {
            try {
                // Start process
                val processBuilder = ProcessBuilder(serverCommand)
                    .redirectErrorStream(false)
                    .directory(File(workspaceRoot))
                val process = processBuilder.start()

                // Create transport
                transport = LspTransport(process) { notification ->
                    handleNotification(notification)
                }

                // Send initialize request
                val initParams = createInitializeParams()
                val response = transport!!.sendRequest("initialize", initParams).get()

                log.info("LSP server initialized: $response")
                isInitialized = true
            } catch (ex: Exception) {
                log.error("Failed to initialize LSP client", ex)
                throw ex
            }
        }
    }

    /**
     * Send initialized notification after initialize request completes.
     */
    fun notifyInitialized() {
        if (isInitialized) {
            transport?.sendNotification("initialized", null)
            log.info("Sent initialized notification")
        }
    }

    /**
     * Notify server about opened document.
     */
    fun didOpen(uri: String, languageId: String, version: Int, text: String) {
        val params = JsonObject().apply {
            val textDocument = JsonObject().apply {
                addProperty("uri", uri)
                addProperty("languageId", languageId)
                addProperty("version", version)
                addProperty("text", text)
            }
            add("textDocument", textDocument)
        }
        transport?.sendNotification("textDocument/didOpen", params)
    }

    /**
     * Notify server about changed document (full sync).
     */
    fun didChange(uri: String, version: Int, text: String) {
        val params = JsonObject().apply {
            val textDocument = JsonObject().apply {
                addProperty("uri", uri)
                addProperty("version", version)
            }
            add("textDocument", textDocument)

            val contentChanges = com.google.gson.JsonArray().apply {
                val change = JsonObject()
                change.addProperty("text", text)
                add(change)
            }
            add("contentChanges", contentChanges)
        }
        transport?.sendNotification("textDocument/didChange", params)
    }

    /**
     * Notify server about closed document.
     */
    fun didClose(uri: String) {
        val params = JsonObject().apply {
            val textDocument = JsonObject().apply {
                addProperty("uri", uri)
            }
            add("textDocument", textDocument)
        }
        transport?.sendNotification("textDocument/didClose", params)
    }

    /**
     * Shutdown and exit.
     */
    override fun close() {
        try {
            if (isInitialized) {
                transport?.sendRequest("shutdown", null)?.get()
                transport?.sendNotification("exit", null)
                log.info("LSP client shutdown")
            }
            transport?.close()
            isInitialized = false
        } catch (ex: Exception) {
            log.error("Error closing LSP client", ex)
        }
    }

    private fun handleNotification(msg: LspIncomingMessage.Notification) {
        when (msg.method) {
            "textDocument/publishDiagnostics" -> {
                // Handled by DiagnosticsHandler (injected via callback)
                log.debug("Received publishDiagnostics: ${msg.params}")
            }
            else -> log.debug("Unhandled notification: ${msg.method}")
        }
    }

    private fun createInitializeParams(): JsonObject {
        return JsonObject().apply {
            addProperty("processId", ProcessHandle.current().pid().toInt())
            addProperty("rootPath", workspaceRoot)

            val rootUri = File(workspaceRoot).toURI().toString()
            addProperty("rootUri", rootUri)

            val capabilities = JsonObject().apply {
                // Text document sync capabilities
                add("textDocument", JsonObject().apply {
                    add("synchronization", JsonObject().apply {
                        addProperty("didSave", true)
                    })
                })
            }
            add("capabilities", capabilities)

            val initOptions = JsonObject().apply {
                // Any ktav-specific options here
            }
            add("initializationOptions", initOptions)
        }
    }
}
