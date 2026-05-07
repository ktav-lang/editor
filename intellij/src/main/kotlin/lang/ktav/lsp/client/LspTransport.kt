package lang.ktav.lsp.client

import com.google.gson.Gson
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.intellij.openapi.diagnostic.Logger
import java.io.*
import java.util.concurrent.BlockingQueue
import java.util.concurrent.CompletableFuture
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.atomic.AtomicInteger

/**
 * JSON-RPC transport over stdio (Language Server Protocol).
 */
class LspTransport(
    private val process: Process,
    private val onNotification: (LspIncomingMessage.Notification) -> Unit
) : AutoCloseable {

    private val log = Logger.getInstance(LspTransport::class.java)
    private val gson = Gson()
    private val nextId = AtomicInteger(1)

    // Pending requests: id → CompletableFuture<JsonElement>
    private val pendingRequests = ConcurrentHashMap<Int, CompletableFuture<JsonElement>>()

    // Use raw streams for binary-safe Content-Length-based protocol
    private val stdin = process.outputStream
    private val stdoutRaw = process.inputStream
    private val stderrRaw = process.errorStream

    private val writeQueue: BlockingQueue<ByteArray> = LinkedBlockingQueue()

    @Volatile
    private var closed = false

    // Reader thread: reads from server stdout
    private val readerThread = Thread({ readLoop() }, "Ktav-LSP-Reader").apply {
        isDaemon = true
        start()
    }

    // Writer thread: writes to server stdin (allows async sends)
    private val writerThread = Thread({ writeLoop() }, "Ktav-LSP-Writer").apply {
        isDaemon = true
        start()
    }

    // Stderr reader thread: collects diagnostic logs from LSP server
    private val stderrThread = Thread({ stderrLoop() }, "Ktav-LSP-Stderr").apply {
        isDaemon = true
        start()
    }

    init {
        log.info("[Ktav Transport] Started threads: reader, writer, stderr")
    }

    /**
     * Send a request and wait for response.
     */
    fun sendRequest(method: String, params: JsonObject?): CompletableFuture<JsonElement> {
        val id = nextId.getAndIncrement()
        val future = CompletableFuture<JsonElement>()
        pendingRequests[id] = future

        val request = LspRequest(id, method, params)
        val json = gson.toJson(request.toJson())
        log.info("[Ktav Transport] >>> Request id=$id method=$method")
        log.debug("[Ktav Transport] >>> Body: $json")
        sendMessage(json)

        // Timeout: 30s
        future.orTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
            .exceptionally { ex ->
                pendingRequests.remove(id)
                log.warn("[Ktav Transport] Request $method (id=$id) timed out or failed: ${ex.message}")
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
        log.info("[Ktav Transport] >>> Notification method=$method")
        log.debug("[Ktav Transport] >>> Body: $json")
        sendMessage(json)
    }

    private fun sendMessage(json: String) {
        if (closed) {
            log.warn("[Ktav Transport] sendMessage on closed transport: $json")
            return
        }
        try {
            val bytes = json.toByteArray(Charsets.UTF_8)
            val header = "Content-Length: ${bytes.size}\r\n\r\n".toByteArray(Charsets.US_ASCII)
            // combine into single buffer for atomic enqueue
            val full = ByteArray(header.size + bytes.size)
            System.arraycopy(header, 0, full, 0, header.size)
            System.arraycopy(bytes, 0, full, header.size, bytes.size)
            writeQueue.offer(full)
        } catch (ex: Exception) {
            log.error("[Ktav Transport] sendMessage failed", ex)
        }
    }

    /**
     * Read messages from server stdout (binary-safe, byte-level).
     */
    private fun readLoop() {
        log.info("[Ktav Transport] Reader thread started")
        try {
            while (!closed) {
                // Read headers (terminated by \r\n\r\n)
                val headers = readHeaders() ?: break

                val contentLengthStr = headers["Content-Length"]
                if (contentLengthStr == null) {
                    log.error("[Ktav Transport] Missing Content-Length header in headers: $headers")
                    break
                }
                val contentLength = contentLengthStr.toIntOrNull()
                if (contentLength == null) {
                    log.error("[Ktav Transport] Invalid Content-Length: $contentLengthStr")
                    break
                }

                // Read message body
                val buffer = ByteArray(contentLength)
                var totalRead = 0
                while (totalRead < contentLength) {
                    val n = stdoutRaw.read(buffer, totalRead, contentLength - totalRead)
                    if (n < 0) {
                        log.error("[Ktav Transport] Unexpected EOF after $totalRead/$contentLength bytes")
                        return
                    }
                    totalRead += n
                }

                val json = String(buffer, Charsets.UTF_8)
                log.debug("[Ktav Transport] <<< Received: $json")
                handleMessage(json)
            }
            log.info("[Ktav Transport] Reader loop exited (closed=$closed)")
        } catch (ex: Exception) {
            if (!closed) {
                log.error("[Ktav Transport] Read loop failed", ex)
            }
        }
    }

    /**
     * Read \r\n-delimited headers until empty line.
     */
    private fun readHeaders(): Map<String, String>? {
        val result = mutableMapOf<String, String>()
        val sb = StringBuilder()
        while (!closed) {
            val ch = stdoutRaw.read()
            if (ch < 0) {
                log.info("[Ktav Transport] EOF on stdout (server exited?)")
                return null
            }
            val c = ch.toChar()
            if (c == '\r') {
                // Expect \n next
                val next = stdoutRaw.read()
                if (next < 0) return null
                if (next.toChar() != '\n') {
                    log.warn("[Ktav Transport] Expected \\n after \\r, got ${next.toChar()}")
                    continue
                }
                val line = sb.toString()
                sb.clear()
                if (line.isEmpty()) {
                    // End of headers
                    return result
                }
                val colonIdx = line.indexOf(':')
                if (colonIdx > 0) {
                    val key = line.substring(0, colonIdx).trim()
                    val value = line.substring(colonIdx + 1).trim()
                    result[key] = value
                }
            } else {
                sb.append(c)
            }
        }
        return null
    }

    /**
     * Write messages to server stdin.
     */
    private fun writeLoop() {
        log.info("[Ktav Transport] Writer thread started")
        try {
            while (!closed) {
                val message = writeQueue.take()
                stdin.write(message)
                stdin.flush()
            }
        } catch (ex: InterruptedException) {
            log.info("[Ktav Transport] Writer interrupted")
        } catch (ex: Exception) {
            if (!closed) {
                log.error("[Ktav Transport] Write loop failed", ex)
            }
        }
        log.info("[Ktav Transport] Writer loop exited")
    }

    /**
     * Read stderr from server (for diagnostic logs).
     */
    private fun stderrLoop() {
        log.info("[Ktav Transport] Stderr thread started")
        try {
            val reader = BufferedReader(InputStreamReader(stderrRaw, Charsets.UTF_8))
            var line: String? = reader.readLine()
            while (line != null && !closed) {
                log.info("[ktav-lsp stderr] $line")
                line = reader.readLine()
            }
            log.info("[Ktav Transport] Stderr loop exited (EOF)")
        } catch (ex: Exception) {
            if (!closed) {
                log.warn("[Ktav Transport] Stderr loop error: ${ex.message}")
            }
        }
    }

    /**
     * Parse and handle incoming message.
     */
    private fun handleMessage(json: String) {
        try {
            val obj = gson.fromJson(json, JsonObject::class.java)

            if (obj.has("id") && (obj.has("result") || obj.has("error"))) {
                // Response
                val id = obj.get("id").asInt
                val result: JsonElement? = obj.get("result")
                val error = if (obj.has("error") && !obj.get("error").isJsonNull) {
                    val errObj = obj.getAsJsonObject("error")
                    LspError(
                        code = errObj.get("code").asInt,
                        message = errObj.get("message").asString,
                        data = errObj.get("data")
                    )
                } else null

                log.info("[Ktav Transport] <<< Response id=$id error=${error?.message}")

                val future = pendingRequests.remove(id)
                if (future != null) {
                    if (error != null) {
                        future.completeExceptionally(Exception("LSP error ${error.code}: ${error.message}"))
                    } else {
                        future.complete(result)
                    }
                } else {
                    log.warn("[Ktav Transport] Response for unknown request id=$id")
                }
            } else if (obj.has("method")) {
                // Notification or server-initiated request
                val method = obj.get("method").asString
                val params: JsonElement? = obj.get("params")

                if (obj.has("id")) {
                    // Server-initiated request — we don't handle these yet
                    log.warn("[Ktav Transport] Server request not handled: $method")
                } else {
                    log.info("[Ktav Transport] <<< Notification method=$method")
                    onNotification(LspIncomingMessage.Notification(method, params))
                }
            } else {
                log.warn("[Ktav Transport] Unrecognized message: $json")
            }
        } catch (ex: Exception) {
            log.error("[Ktav Transport] Failed to parse: $json", ex)
        }
    }

    override fun close() {
        if (closed) return
        closed = true
        log.info("[Ktav Transport] Closing transport")
        try {
            stdin.close()
        } catch (_: Exception) {}
        try {
            stdoutRaw.close()
        } catch (_: Exception) {}
        try {
            stderrRaw.close()
        } catch (_: Exception) {}
        try {
            process.destroy()
            process.waitFor(2, java.util.concurrent.TimeUnit.SECONDS)
            if (process.isAlive) {
                process.destroyForcibly()
            }
        } catch (ex: Exception) {
            log.warn("[Ktav Transport] Process destroy error: ${ex.message}")
        }
        readerThread.interrupt()
        writerThread.interrupt()
        stderrThread.interrupt()
        log.info("[Ktav Transport] Closed")
    }
}
