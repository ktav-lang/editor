package lang.ktav.lsp.client

import com.google.gson.Gson
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.intellij.openapi.diagnostic.Logger
import java.io.*
import java.util.concurrent.BlockingQueue
import java.util.concurrent.CompletableFuture
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.atomic.AtomicInteger

/**
 * JSON-RPC transport over stdio (Language Server Protocol).
 *
 * Handles:
 * - Message framing with Content-Length header
 * - Request/response matching via id
 * - Notifications (one-way messages)
 * - Bidirectional communication: send requests/notifications, receive responses/notifications
 */
class LspTransport(
    private val process: Process,
    private val onNotification: (LspIncomingMessage.Notification) -> Unit
) : AutoCloseable {

    private val log = Logger.getInstance(LspTransport::class.java)
    private val gson = Gson()
    private val nextId = AtomicInteger(1)

    // Pending requests: id → CompletableFuture<JsonElement>
    private val pendingRequests = mutableMapOf<Int, CompletableFuture<JsonElement>>()

    // Reader thread: reads from server stdout
    private val readerThread = Thread { readLoop() }.apply {
        isDaemon = true
        start()
    }

    // Writer thread: writes to server stdin (allows async sends)
    private val writeQueue: BlockingQueue<String> = LinkedBlockingQueue()
    private val writerThread = Thread { writeLoop() }.apply {
        isDaemon = true
        start()
    }

    private val stdin = BufferedWriter(OutputStreamWriter(process.outputStream, Charsets.UTF_8))
    private val stdout = BufferedReader(InputStreamReader(process.inputStream, Charsets.UTF_8))
    private val stderr = BufferedReader(InputStreamReader(process.errorStream, Charsets.UTF_8))

    /**
     * Send a request and wait for response.
     * Returns CompletableFuture that completes when server responds.
     */
    fun sendRequest(method: String, params: JsonObject?): CompletableFuture<JsonElement> {
        val id = nextId.getAndIncrement()
        val future = CompletableFuture<JsonElement>()
        pendingRequests[id] = future

        val request = LspRequest(id, method, params)
        val json = gson.toJson(request.toJson())
        sendMessage(json)

        // Timeout: if no response in 30s, fail the future
        future.orTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
            .exceptionally { ex ->
                pendingRequests.remove(id)
                log.warn("LSP request $method ($id) timed out")
                null
            }

        return future
    }

    /**
     * Send a notification (no response expected).
     */
    fun sendNotification(method: String, params: JsonObject?) {
        val notification = LspNotification(method, params)
        val json = gson.toJson(notification.toJson())
        sendMessage(json)
    }

    private fun sendMessage(json: String) {
        val bytes = json.toByteArray(Charsets.UTF_8)
        val header = "Content-Length: ${bytes.size}\r\n\r\n"
        writeQueue.offer(header + json)
    }

    /**
     * Read messages from server stdin.
     * Messages are framed with Content-Length header.
     */
    private fun readLoop() {
        try {
            val headers = mutableMapOf<String, String>()
            while (true) {
                // Read headers until empty line
                headers.clear()
                var line = stdout.readLine() ?: break // EOF
                while (line.isNotEmpty()) {
                    val (key, value) = line.split(": ", limit = 2)
                    headers[key] = value
                    line = stdout.readLine() ?: break
                }

                val contentLength = headers["Content-Length"]?.toIntOrNull()
                    ?: throw IOException("Missing Content-Length header")

                // Read message body
                val buffer = CharArray(contentLength)
                val read = stdout.read(buffer)
                if (read != contentLength) {
                    throw IOException("Expected $contentLength bytes, got $read")
                }

                val json = String(buffer)
                handleMessage(json)
            }
        } catch (ex: Exception) {
            log.error("LSP transport read loop failed", ex)
        }
    }

    /**
     * Write messages to server stdout.
     */
    private fun writeLoop() {
        try {
            while (true) {
                val message = writeQueue.take()
                stdin.write(message)
                stdin.flush()
            }
        } catch (ex: Exception) {
            log.error("LSP transport write loop failed", ex)
        }
    }

    /**
     * Parse and handle incoming message.
     */
    private fun handleMessage(json: String) {
        try {
            val obj = gson.fromJson(json, JsonObject::class.java)

            // Response to our request?
            if (obj.has("id")) {
                val id = obj.get("id").asInt
                val result = obj.get("result")
                val error = if (obj.has("error")) {
                    val errObj = obj.getAsJsonObject("error")
                    LspError(
                        code = errObj.get("code").asInt,
                        message = errObj.get("message").asString,
                        data = errObj.get("data")
                    )
                } else null

                val future = pendingRequests.remove(id)
                if (future != null) {
                    if (error != null) {
                        future.completeExceptionally(Exception("LSP error: ${error.message}"))
                    } else {
                        future.complete(result)
                    }
                } else {
                    log.warn("Received response for unknown request id=$id")
                }
            } else {
                // Notification from server
                val method = obj.get("method").asString
                val params = obj.get("params")
                onNotification(LspIncomingMessage.Notification(method, params))
            }
        } catch (ex: Exception) {
            log.error("Failed to parse LSP message: $json", ex)
        }
    }

    override fun close() {
        try {
            stdin.close()
            stdout.close()
            stderr.close()
            process.destroy()
            process.waitFor()
        } catch (ex: Exception) {
            log.error("Error closing LSP transport", ex)
        }
    }
}
