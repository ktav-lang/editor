package lang.ktav.lsp.client

import com.google.gson.JsonElement
import com.google.gson.JsonObject

/**
 * JSON-RPC message structures for LSP communication.
 */

/** LSP Request: method + params, expects response */
data class LspRequest(
    val id: Int,
    val method: String,
    val params: JsonElement? = null
) {
    fun toJson(): JsonObject {
        val obj = JsonObject()
        obj.addProperty("jsonrpc", "2.0")
        obj.addProperty("id", id)
        obj.addProperty("method", method)
        if (params != null) {
            obj.add("params", params)
        }
        return obj
    }
}

/** LSP Notification: method + params, no response expected */
data class LspNotification(
    val method: String,
    val params: JsonElement? = null
) {
    fun toJson(): JsonObject {
        val obj = JsonObject()
        obj.addProperty("jsonrpc", "2.0")
        obj.addProperty("method", method)
        if (params != null) {
            obj.add("params", params)
        }
        return obj
    }
}

/** LSP Response: result or error for request id */
data class LspResponse(
    val id: Int,
    val result: JsonElement? = null,
    val error: LspError? = null
)

/** LSP Error object */
data class LspError(
    val code: Int,
    val message: String,
    val data: JsonElement? = null
)

/** Incoming message from server (can be response or notification) */
sealed class LspIncomingMessage {
    data class Response(val id: Int, val result: JsonElement?, val error: LspError?) : LspIncomingMessage()
    data class Notification(val method: String, val params: JsonElement?) : LspIncomingMessage()
}
