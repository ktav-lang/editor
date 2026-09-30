package lang.ktav.lsp.formatting

import com.google.gson.JsonElement
import com.google.gson.JsonParser
import com.intellij.formatting.FormattingContext
import com.intellij.formatting.service.AsyncFormattingRequest
import com.intellij.openapi.editor.impl.DocumentImpl
import com.intellij.openapi.progress.EmptyProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.util.TextRange
import com.intellij.testFramework.TestApplicationManager
import lang.ktav.lsp.KtavFormattingService
import lang.ktav.lsp.lifecycle.RecordingSyncClient
import lang.ktav.lsp.lifecycle.TestProject
import lang.ktav.lsp.lifecycle.edit
import lang.ktav.lsp.lifecycle.withCurrentOnEdt
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Test
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource
import java.io.File
import java.util.concurrent.CompletableFuture
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class KtavFormattingServiceTest {
    companion object {
        @BeforeAll
        @JvmStatic
        fun initializeApplication() { TestApplicationManager.getInstance() }
    }

    private val root = Disposer.newDisposable()
    private val uri = "file:///shared.ktav"
    private val formatter = KtavFormattingService()
    private val replacement = JsonParser.parseString(
        """[{"range":{"start":{"line":0,"character":0},"end":{"line":1,"character":0}},"newText":"v: 2\n"}]"""
    )

    @AfterEach
    fun cleanup() = Disposer.dispose(root)

    @ParameterizedTest
    @ValueSource(booleans = [false, true])
    fun `formatting a shared file uses the updated owner cache in either project order`(reverse: Boolean) {
        val owners = listOf(TestProject(root, "A"), TestProject(root, "B"))
        val clients = listOf(RecordingSyncClient(), RecordingSyncClient())
        val document = DocumentImpl("v: 1\n")
        val order = if (reverse) listOf(1, 0) else listOf(0, 1)
        order.forEach { owners[it].tracker.attach(uri, document, clients[it].connection) }
        document.edit("v:2\n")
        val target = order.last()
        val request = Request(document.text) { document.edit(it) }
        val snapshot = owners[target].tracker.snapshot(uri, document, clients[target], request.documentText)!!
        val task = formatter.guardedTask(request, request.documentText, {
            assertEquals("v:2\n", clients[target].documents[uri])
            CompletableFuture.completedFuture(replacement)
        }) { action ->
            snapshot.withCurrentOnEdt(action)
        }
        task.run()
        assertEquals(listOf("v: 2\n"), request.ready)
        assertTrue(request.errors.isEmpty())
        assertEquals("v: 2\n", document.text)
        clients.forEach { assertEquals(document.text, it.documents[uri]) }
    }

    @ParameterizedTest
    @ValueSource(strings = ["edits", "empty", "null", "non-array", "java-null"])
    fun `stale responses never deliver replacement text including empty and null results`(kind: String) {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v:1\n")
        owner.tracker.attach(uri, document, client.connection)
        val request = Request(document.text)
        val snapshot = owner.tracker.snapshot(uri, document, client, request.documentText)!!
        val result = when (kind) {
            "edits" -> replacement
            "empty" -> JsonParser.parseString("[]")
            "null" -> JsonParser.parseString("null")
            "non-array" -> JsonParser.parseString("{}")
            else -> null
        }
        val task = formatter.guardedTask(request, request.documentText, {
            Response(result) { document.edit("v: user-edit\n") }
        }) { action ->
            snapshot.withCurrentOnEdt(action)
        }
        task.run()
        assertTrue(request.ready.isEmpty())
        assertEquals(1, request.errors.size)
        assertEquals("v: user-edit\n", document.text)
    }

    @Test
    fun `document changes before dispatch prevent the server request`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v:1\n")
        owner.tracker.attach(uri, document, client.connection)
        val request = Request(document.text)
        val snapshot = owner.tracker.snapshot(uri, document, client, request.documentText)!!
        val task = formatter.guardedTask(request, request.documentText, { error("Must not send stale request") }) { action ->
            snapshot.withCurrentOnEdt(action)
        }
        document.edit("v: new\n")
        task.run()
        assertTrue(request.ready.isEmpty())
        assertEquals(1, request.errors.size)
    }

    @ParameterizedTest
    @ValueSource(strings = ["close", "reopen", "dispose", "replace-client"])
    fun `owner lifecycle changes discard in-flight responses while the other project stays synced`(kind: String) {
        val owner = TestProject(root, "owner")
        val other = TestProject(root, "other")
        val client = RecordingSyncClient()
        val otherClient = RecordingSyncClient()
        val document = DocumentImpl("v:1\n")
        owner.tracker.attach(uri, document, client.connection)
        other.tracker.attach(uri, document, otherClient.connection)
        val request = Request(document.text)
        val snapshot = owner.tracker.snapshot(uri, document, client, request.documentText)!!
        val task = formatter.guardedTask(request, request.documentText, {
            Response(replacement) {
                when (kind) {
                    "dispose" -> Disposer.dispose(owner.project)
                    "replace-client" -> owner.tracker.attach(uri, document, RecordingSyncClient().connection)
                    else -> {
                        owner.tracker.detach(uri)
                        if (kind == "reopen") owner.tracker.attach(uri, document, client.connection)
                    }
                }
            }
        }) { action ->
            snapshot.withCurrentOnEdt(action)
        }
        task.run()
        assertTrue(request.ready.isEmpty())
        assertEquals(1, request.errors.size)
        document.edit("v: surviving-edit\n")
        assertEquals(document.text, otherClient.documents[uri])
    }

    @Test
    fun `cancellation after response but before delivery is checked again`() {
        val request = Request("v:1\n")
        var guardedCalls = 0
        lateinit var task: KtavFormattingService.GuardedTask
        task = formatter.guardedTask(request, request.documentText, {
            CompletableFuture.completedFuture(replacement)
        }) { action ->
            if (++guardedCalls == 2) task.cancel()
            action()
            true
        }
        task.run()
        assertEquals(2, guardedCalls)
        assertTrue(request.ready.isEmpty())
        assertTrue(request.errors.isEmpty())
    }

    @ParameterizedTest
    @ValueSource(strings = ["[]", "null", "{}", "java-null"])
    fun `unchanged current responses complete with the original snapshot`(json: String) {
        val request = Request("v: 1\n")
        formatter.guardedTask(request, request.documentText, {
            CompletableFuture.completedFuture(if (json == "java-null") null else JsonParser.parseString(json))
        }) { action -> action(); true }.run()
        assertEquals(listOf("v: 1\n"), request.ready)
        assertTrue(request.errors.isEmpty())
    }

    @ParameterizedTest
    @ValueSource(strings = ["edit", "close", "reopen", "dispose", "replace-client", "inactive", "cancel"])
    fun `queued delivery rechecks ownership and text at application time`(kind: String) {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v:1\n")
        owner.tracker.attach(uri, document, client.connection)
        val request = Request(document.text) { document.edit(it) }
        val snapshot = owner.tracker.snapshot(uri, document, client, request.documentText)!!
        val queued = mutableListOf<() -> Unit>()
        val task = formatter.guardedTask(request, request.documentText, {
            CompletableFuture.completedFuture(replacement)
        }, dispatchResult = { queued += it }) { action ->
            snapshot.withCurrentOnEdt(action)
        }
        task.run()
        assertTrue(request.ready.isEmpty())
        assertEquals(1, queued.size)
        when (kind) {
            "edit" -> document.edit("v: queued-user-edit\n")
            "dispose" -> Disposer.dispose(owner.project)
            "replace-client" -> owner.tracker.attach(uri, document, RecordingSyncClient().connection)
            "inactive" -> client.active = false
            "cancel" -> task.cancel()
            else -> {
                owner.tracker.detach(uri)
                if (kind == "reopen") owner.tracker.attach(uri, document, client.connection)
            }
        }
        val text = document.text
        queued.single().invoke()
        assertTrue(request.ready.isEmpty())
        assertEquals(if (kind == "cancel") 0 else 1, request.errors.size)
        assertEquals(text, document.text)
    }

    @Test
    fun `progress cancellation before its platform cancel callback rejects queued text`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v:1\n")
        owner.tracker.attach(uri, document, client.connection)
        val request = Request(document.text) { document.edit(it) }
        val snapshot = owner.tracker.snapshot(uri, document, client, document.text)!!
        val queued = mutableListOf<() -> Unit>()
        val indicator = EmptyProgressIndicator()
        val task = formatter.guardedTask(request, document.text, {
            CompletableFuture.completedFuture(replacement)
        }, dispatchResult = { queued += it }) { action -> snapshot.withCurrentOnEdt(action) }
        ProgressManager.getInstance().runProcess(Runnable { task.run() }, indicator)
        // AsyncDocumentFormattingService's onCancel callback need not have run yet.
        indicator.cancel()
        queued.single().invoke()
        assertEquals("v:1\n", document.text)
        assertEquals(listOf("open"), client.messages.map { it.kind })
        assertTrue(request.ready.isEmpty())
        assertTrue(request.errors.isEmpty())
    }

    @Test
    fun `two queued responses cannot overwrite the first applied result`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v:1\n")
        owner.tracker.attach(uri, document, client.connection)
        val requests = listOf(Request(document.text) { document.edit(it) }, Request(document.text) { document.edit(it) })
        val queued = mutableListOf<() -> Unit>()
        for (request in requests) {
            val snapshot = owner.tracker.snapshot(uri, document, client, request.documentText)!!
            formatter.guardedTask(request, request.documentText, {
                CompletableFuture.completedFuture(replacement)
            }, dispatchResult = { queued += it }) { action ->
                snapshot.withCurrentOnEdt(action)
            }.run()
        }
        assertEquals(2, queued.size)
        queued[0]()
        queued[1]()
        assertEquals(listOf("v: 2\n"), requests[0].ready)
        assertTrue(requests[1].ready.isEmpty())
        assertEquals(1, requests[1].errors.size)
        assertEquals("v: 2\n", document.text)
        assertEquals(listOf(1, 2), client.messages.map { it.version })
    }

    @Test
    fun `cancel before dispatch does not contact the server or complete the request`() {
        val request = Request("v:1\n")
        val task = formatter.guardedTask(request, request.documentText, { error("Cancelled request") }) {
            error("Cancelled snapshot")
        }
        assertTrue(task.cancel())
        task.run()
        assertTrue(request.ready.isEmpty())
        assertTrue(request.errors.isEmpty())
    }

    @Test
    fun `expired queued delivery cannot write after the platform request timeout`() {
        val request = Request("v:1\n")
        val queued = mutableListOf<() -> Unit>()
        var now = 0L
        formatter.guardedTask(request, request.documentText, {
            CompletableFuture.completedFuture(replacement)
        }, dispatchResult = { queued += it }, nanoTime = { now }) { action -> action(); true }.run()
        now = TimeUnit.SECONDS.toNanos(30)
        queued.single().invoke()
        assertTrue(request.ready.isEmpty())
        assertEquals(1, request.errors.size)
    }

    @Test
    fun `failed formatting response reports an error without changing the document`() {
        val request = Request("v:1\n")
        formatter.guardedTask(request, request.documentText, {
            CompletableFuture.failedFuture(IllegalStateException("Formatting failed"))
        }) { action -> action(); true }.run()
        assertTrue(request.ready.isEmpty())
        assertEquals(1, request.errors.size)
    }

    @Test
    fun `real in-flight formatting response cannot replace a later edit`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v:1\n")
        owner.tracker.attach(uri, document, client.connection)
        val request = Request(document.text) { document.edit(it) }
        val snapshot = owner.tracker.snapshot(uri, document, client, request.documentText)!!
        val pending = CompletableFuture<JsonElement>()
        val sent = CountDownLatch(1)
        val task = formatter.guardedTask(request, request.documentText, {
            sent.countDown()
            pending
        }) { action -> snapshot.withCurrentOnEdt(action) }
        val running = CompletableFuture.runAsync { task.run() }
        try {
            assertTrue(sent.await(5, TimeUnit.SECONDS))
            document.edit("v: real-user-edit\n")
            pending.complete(replacement)
            running.get(5, TimeUnit.SECONDS)
            assertTrue(request.ready.isEmpty())
            assertEquals(1, request.errors.size)
            assertEquals("v: real-user-edit\n", document.text)
            assertEquals(document.text, client.documents[uri])
        } finally {
            task.cancel()
            pending.complete(replacement)
            running.get(5, TimeUnit.SECONDS)
        }
    }

    @Test
    fun `cancelling a pending response releases the task without delivering text or errors`() {
        val request = Request("v:1\n")
        val pending = CompletableFuture<JsonElement>()
        val sent = CountDownLatch(1)
        val task = formatter.guardedTask(request, request.documentText, {
            sent.countDown()
            pending
        }) { action -> action(); true }
        val running = CompletableFuture.runAsync { task.run() }
        try {
            assertTrue(sent.await(5, TimeUnit.SECONDS))
            assertTrue(task.cancel())
            running.get(5, TimeUnit.SECONDS)
            assertTrue(pending.isCancelled)
            assertTrue(request.ready.isEmpty())
            assertTrue(request.errors.isEmpty())
        } finally {
            task.cancel()
            pending.complete(replacement)
            running.get(5, TimeUnit.SECONDS)
        }
    }

    private class Response(private val result: JsonElement?, private val beforeResult: () -> Unit) : CompletableFuture<JsonElement>() {
        override fun get(timeout: Long, unit: TimeUnit): JsonElement? {
            beforeResult()
            return result
        }
    }

    private class Request(private val text: String, private val apply: (String) -> Unit = {}) : AsyncFormattingRequest {
        val ready = mutableListOf<String>()
        val errors = mutableListOf<String>()
        override fun getDocumentText(): String = text
        override fun getIOFile(): File = File("shared.ktav")
        override fun getFormattingRanges(): List<TextRange> = listOf(TextRange(0, text.length))
        override fun canChangeWhitespaceOnly(): Boolean = false
        override fun isQuickFormat(): Boolean = false
        override fun getContext(): FormattingContext = error("Context not used by task")
        override fun onTextReady(text: String?) { checkNotNull(text); ready += text; apply(text) }
        override fun onError(title: String, message: String) { errors += message }
        override fun onError(title: String, message: String, offset: Int) = onError(title, message)
    }
}
