package lang.ktav.lsp

import com.intellij.formatting.FormattingContext
import com.intellij.formatting.service.AsyncFormattingRequest
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.command.undo.UndoManager
import com.intellij.openapi.fileEditor.impl.text.TextEditorProvider
import com.intellij.openapi.util.TextRange
import com.intellij.application.options.CodeStyle
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.runInEdtAndWait
import com.intellij.util.ui.UIUtil
import lang.ktav.KtavFileType
import lang.ktav.lsp.client.KtavLspClient
import lang.ktav.lsp.lifecycle.ChangeTracker
import lang.ktav.lsp.lifecycle.ProjectClientLifecycle
import lang.ktav.lsp.lifecycle.getLspService
import java.io.File
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit

/** Exercises the actual platform request, createFormattingTask, write, callback and undo path. */
class KtavFormattingPlatformTest : LocalPlatformTestCase() {
    private inline fun withServer(action: (TestLspProcess) -> Unit) {
        TestLspProcess().use { server ->
            try {
                action(server)
            } finally {
                project.getLspService().close()
            }
        }
    }

    fun testFormattingHasItsOwnUndoStepAndSynchronizesTheAppliedText() {
        withServer { server ->
            val client = installClient(server)
            myFixture.configureByText(KtavFileType, "v:0\n")
            val document = myFixture.editor.document
            val uri = UriUtil.fromVirtualFile(myFixture.file.virtualFile)
            val tracker = ChangeTracker.getInstance(project)
            tracker.attachIfNeeded(myFixture.file.virtualFile, document, client)
            WriteCommandAction.runWriteCommandAction(project, "User edit", null, Runnable { document.setText("v:1\n") })
            server.releaseFormatting()
            formatDocument("v: 1\n")
            assertEquals("v: 1\n", document.text)
            assertEquals("v:1\n", server.awaitFile("format.received"))
            assertEquals("v: 1\n", server.awaitText("change.received", "v: 1\n"))
            assertNotNull(tracker.snapshot(uri, document, client, document.text))

            val fileEditor = TextEditorProvider.getInstance().getTextEditor(myFixture.editor)
            val undo = UndoManager.getInstance(project)
            assertTrue(undo.isUndoAvailable(fileEditor))
            undo.undo(fileEditor)
            assertEquals("v:1\n", document.text)
            assertNotNull(tracker.snapshot(uri, document, client, document.text))
            undo.redo(fileEditor)
            assertEquals("v: 1\n", document.text)
            assertNotNull(tracker.snapshot(uri, document, client, document.text))
        }
    }

    fun testActualPlatformResponseCannotOverwriteAnEdit() = staleResponse("edit")
    fun testActualPlatformResponseCannotApplyAfterCloseReopen() = staleResponse("reopen")
    fun testActualPlatformResponseCannotApplyAfterClientDisposal() = staleResponse("client-close")

    private fun staleResponse(transition: String) {
        withServer { server ->
            val client = installClient(server)
            myFixture.configureByText(KtavFileType, "v:1\n")
            val document = myFixture.editor.document
            val file = myFixture.file.virtualFile
            val tracker = ChangeTracker.getInstance(project)
            tracker.attachIfNeeded(file, document, client)
            val request = ObservedRequest(document.text, FormattingContext.create(myFixture.file, CodeStyle.getSettings(myFixture.file)))
            val createTask = KtavFormattingService::class.java
                .getDeclaredMethod("createFormattingTask", AsyncFormattingRequest::class.java)
                .apply { isAccessible = true }
            val task = checkNotNull(createTask.invoke(KtavFormattingService(), request)) as Runnable
            val formatting = CompletableFuture.runAsync { task.run() }
            assertEquals("v:1\n", server.awaitFile("format.received"))
            when (transition) {
                "edit" -> WriteCommandAction.runWriteCommandAction(project, Runnable { document.setText("v:user\n") })
                "reopen" -> { tracker.detach(file); tracker.attachIfNeeded(file, document, client) }
                else -> project.getLspService().close()
            }
            val retained = document.text
            server.releaseFormatting()
            awaitFormatting(formatting) { request.completion.isDone }
            assertNull(request.ready)
            assertEquals(1, request.errors.size)
            assertEquals(retained, document.text)
        }
    }

    private fun installClient(server: TestLspProcess): KtavLspClient {
        val service = project.getLspService()
        // Keep the real service and real lifecycle; only substitute OS child creation/discovery.
        // No mock can supply the Document, platform request, snapshot, transport or response.
        val lifecycleField = service.javaClass.getDeclaredField("lifecycle").apply { isAccessible = true }
        (lifecycleField.get(service) as AutoCloseable).close()
        val client = server.client()
        val lifecycle = ProjectClientLifecycle(
            { project.isDisposed }, { client }, KtavLspClient::initialize, KtavLspClient::notifyInitialized
        ) { throw AssertionError(it) }
        lifecycleField.set(service, lifecycle)
        CompletableFuture.runAsync { service.ensureInitialized() }.get(10, TimeUnit.SECONDS)
        assertSame(client, service.getClient())
        return client
    }

    private fun formatDocument(expected: String) {
        val context = FormattingContext.create(myFixture.file, CodeStyle.getSettings(myFixture.file))
        val document = myFixture.editor.document
        runInEdtAndWait {
            KtavFormattingService().formatDocument(document, listOf(context.formattingRange), context, false, false)
        }
        awaitFormatting(CompletableFuture.completedFuture(null)) { document.text == expected }
    }

    private fun awaitFormatting(formatting: CompletableFuture<Void>, delivered: () -> Boolean) {
        val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(15)
        while (!formatting.isDone || !delivered()) {
            UIUtil.dispatchAllInvocationEvents()
            check(System.nanoTime() < deadline) { "Platform formatting did not complete" }
            Thread.sleep(10)
        }
        formatting.get(1, TimeUnit.SECONDS)
        PlatformTestUtil.dispatchAllEventsInIdeEventQueue()
    }

    private class ObservedRequest(private val originalText: String, private val context: FormattingContext) : AsyncFormattingRequest {
        val completion = CompletableFuture<Unit>()
        var ready: String? = null
            private set
        val errors = mutableListOf<String>()
        override fun getDocumentText(): String = originalText
        override fun getIOFile(): File = File("platform.ktav")
        override fun getFormattingRanges(): List<TextRange> = listOf(context.formattingRange)
        override fun canChangeWhitespaceOnly(): Boolean = false
        override fun isQuickFormat(): Boolean = false
        override fun getContext(): FormattingContext = context
        override fun onTextReady(text: String?) { ready = checkNotNull(text); completion.complete(Unit) }
        override fun onError(title: String, message: String) { errors += message; completion.complete(Unit) }
        override fun onError(title: String, message: String, offset: Int) = onError(title, message)
    }
}
