package lang.ktav.lsp.client

import com.google.gson.JsonObject
import com.intellij.openapi.diagnostic.Logger
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
    private val workspaceRoot: String,
    private val onDiagnostics: (com.google.gson.JsonElement?) -> Unit = {},
    private val onPostDiagnostics: () -> Unit = {}
) : AutoCloseable {

    private val log = Logger.getInstance(KtavLspClient::class.java)
    private var transport: LspTransport? = null
    @Volatile private var isInitialized = false
    private var process: Process? = null

    /**
     * Start the LSP server process and send initialize request.
     */
    fun initialize(): CompletableFuture<Void> {
        return CompletableFuture.runAsync {
            try {
                log.info("[Ktav LSP Client] Starting process: $serverCommand")
                log.info("[Ktav LSP Client] Working directory: $workspaceRoot")

                // Start process
                val processBuilder = ProcessBuilder(serverCommand)
                    .redirectErrorStream(false)
                    .directory(File(workspaceRoot))
                process = processBuilder.start()

                log.info("[Ktav LSP Client] Process started, PID=${process?.pid()}")

                // Create transport
                transport = LspTransport(
                    process = process!!,
                    onNotification = { notification ->
                        handleNotification(notification)
                    }
                )
                log.info("[Ktav LSP Client] Transport created, sending initialize request")

                // Send initialize request
                val initParams = createInitializeParams()
                log.info("[Ktav LSP Client] Initialize params: $initParams")

                val response = transport!!.sendRequest("initialize", initParams).get()

                log.info("[Ktav LSP Client] Initialize response received: $response")
                isInitialized = true
            } catch (ex: Exception) {
                log.error("[Ktav LSP Client] Failed to initialize", ex)
                throw ex
            }
        }
    }

    /**
     * Send initialized notification after initialize request completes.
     */
    fun notifyInitialized() {
        if (isInitialized) {
            log.info("[Ktav LSP Client] Sending 'initialized' notification")
            transport?.sendNotification("initialized", JsonObject())
        } else {
            log.warn("[Ktav LSP Client] Cannot send 'initialized': not initialized yet")
        }
    }

    /**
     * Notify server about opened document.
     */
    fun didOpen(uri: String, languageId: String, version: Int, text: String) {
        if (!isInitialized) {
            log.warn("[Ktav LSP Client] didOpen ignored: not initialized")
            return
        }
        log.info("[Ktav LSP Client] didOpen: uri=$uri, version=$version, textLength=${text.length}")
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
        if (!isInitialized) {
            log.warn("[Ktav LSP Client] didChange ignored: not initialized")
            return
        }
        log.debug("[Ktav LSP Client] didChange: uri=$uri, version=$version, textLength=${text.length}")
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
        if (!isInitialized) {
            log.warn("[Ktav LSP Client] didClose ignored: not initialized")
            return
        }
        log.info("[Ktav LSP Client] didClose: uri=$uri")
        val params = JsonObject().apply {
            val textDocument = JsonObject().apply {
                addProperty("uri", uri)
            }
            add("textDocument", textDocument)
        }
        transport?.sendNotification("textDocument/didClose", params)
    }

    /**
     * Send `textDocument/formatting` request and return `TextEdit[]` JSON.
     * Returns null if not initialized.
     */
    fun sendFormatting(params: JsonObject): java.util.concurrent.CompletableFuture<com.google.gson.JsonElement> {
        if (!isInitialized) {
            log.warn("[Ktav LSP Client] formatting ignored: not initialized")
            return java.util.concurrent.CompletableFuture.failedFuture(IllegalStateException("LSP not initialized"))
        }
        log.info("[Ktav LSP Client] formatting request")
        return transport!!.sendRequest("textDocument/formatting", params)
    }

    /**
     * Shutdown and exit.
     */
    override fun close() {
        try {
            log.info("[Ktav LSP Client] Closing client")
            if (isInitialized) {
                try {
                    transport?.sendRequest("shutdown", null)?.get(2, java.util.concurrent.TimeUnit.SECONDS)
                } catch (ex: Exception) {
                    log.warn("[Ktav LSP Client] shutdown request failed: ${ex.message}")
                }
                try {
                    transport?.sendNotification("exit", null)
                } catch (ex: Exception) {
                    log.warn("[Ktav LSP Client] exit notification failed: ${ex.message}")
                }
            }
            transport?.close()
            isInitialized = false
            log.info("[Ktav LSP Client] Closed successfully")
        } catch (ex: Exception) {
            log.error("[Ktav LSP Client] Error during close", ex)
        }
    }

    private fun handleNotification(msg: LspIncomingMessage.Notification) {
        log.debug("[Ktav LSP Client] Received notification: ${msg.method}")
        when (msg.method) {
            "textDocument/publishDiagnostics" -> {
                log.info("[Ktav LSP Client] publishDiagnostics received: ${msg.params}")
                try {
                    onDiagnostics(msg.params)
                    onPostDiagnostics()
                } catch (ex: Exception) {
                    log.warn("[Ktav LSP Client] Failed to handle publishDiagnostics", ex)
                }
            }
            "window/logMessage" -> {
                val params = msg.params?.asJsonObject ?: return
                val message = params.get("message")?.asString ?: return
                val type = params.get("type")?.asInt ?: 4
                log.info("[Ktav LSP Server log type=$type]: $message")
            }
            "window/showMessage" -> {
                val params = msg.params?.asJsonObject ?: return
                val message = params.get("message")?.asString ?: return
                log.info("[Ktav LSP Server message]: $message")
            }
            else -> log.info("[Ktav LSP Client] Unhandled notification: ${msg.method}")
        }
    }

    private fun createInitializeParams(): JsonObject {
        return JsonObject().apply {
            addProperty("processId", ProcessHandle.current().pid().toInt())
            addProperty("rootPath", workspaceRoot)

            val rootUri = File(workspaceRoot).toURI().toString()
            addProperty("rootUri", rootUri)

            val capabilities = JsonObject().apply {
                add("textDocument", JsonObject().apply {
                    add("synchronization", JsonObject().apply {
                        addProperty("didSave", true)
                        addProperty("dynamicRegistration", false)
                    })
                    add("publishDiagnostics", JsonObject().apply {
                        addProperty("relatedInformation", true)
                    })
                })
            }
            add("capabilities", capabilities)

            add("initializationOptions", JsonObject())
        }
    }
}
