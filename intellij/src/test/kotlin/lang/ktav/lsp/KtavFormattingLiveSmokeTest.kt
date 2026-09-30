package lang.ktav.lsp

import com.intellij.formatting.FormattingContext
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.application.options.CodeStyle
import com.intellij.testFramework.runInEdtAndWait
import com.intellij.util.ui.UIUtil
import lang.ktav.KtavFileType
import lang.ktav.lsp.lifecycle.ChangeTracker
import lang.ktav.lsp.lifecycle.getLspService
import lang.ktav.lsp.settings.KtavSettings
import java.io.File
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit

/** Opt-in smoke against the actual ktav-lsp binary, including discovery and platform callbacks. */
class KtavFormattingLiveSmokeTest : LocalPlatformTestCase() {
    fun testLiveServerFormattingUsesChangesAndReopenedDocument() {
        val binary = checkNotNull(System.getenv("KTAV_LSP_SMOKE_BINARY")?.takeIf { it.isNotBlank() }) {
            "Set KTAV_LSP_SMOKE_BINARY to run the live IntelliJ/LSP smoke"
        }
        check(File(binary).isFile) { "KTAV_LSP_SMOKE_BINARY is not a file" }
        val settings = KtavSettings.getInstance().state
        val previousPath = settings.serverPath
        settings.serverPath = binary
        val service = project.getLspService()
        try {
            CompletableFuture.runAsync { service.ensureInitialized() }.get(20, TimeUnit.SECONDS)
            val client = checkNotNull(service.getClient()) { "Live server did not initialize" }
            myFixture.configureByText(KtavFileType, "items: [\nname: (value)\n]\n")
            val file = myFixture.file.virtualFile
            val document = myFixture.editor.document
            val tracker = ChangeTracker.getInstance(project)
            tracker.attachIfNeeded(file, document, client)
            format("items: [\n    name: (value)\n]\n")
            assertEquals("items: [\n    name: (value)\n]\n", document.text)

            // Different server-visible content distinguishes didChange from formatting the old didOpen cache.
            WriteCommandAction.runWriteCommandAction(project, Runnable {
                document.setText("items: [\nname: ((value))\n]\n")
            })
            format("items: [\n    name: ((value))\n]\n")
            assertEquals("items: [\n    name: ((value))\n]\n", document.text)
            tracker.detach(file)
            WriteCommandAction.runWriteCommandAction(project, Runnable {
                document.setText("items: [\ntrue\n]\n")
            })
            tracker.attachIfNeeded(file, document, client)
            format("items: [\n    true\n]\n")
            assertEquals("items: [\n    true\n]\n", document.text)
            assertNotNull(tracker.snapshot(UriUtil.fromVirtualFile(file), document, client, document.text))
            println("KTAV_LIVE_SMOKE: actual platform formatting applied didOpen/didChange/reopen content")
        } finally {
            service.close()
            settings.serverPath = previousPath
        }
    }

    private fun format(expected: String) {
        val context = FormattingContext.create(myFixture.file, CodeStyle.getSettings(myFixture.file))
        val document = myFixture.editor.document
        runInEdtAndWait {
            KtavFormattingService().formatDocument(document, listOf(context.formattingRange), context, false, false)
        }
        val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(20)
        while (document.text != expected) {
            UIUtil.dispatchAllInvocationEvents()
            check(System.nanoTime() < deadline) { "Live platform formatting did not complete" }
            Thread.sleep(10)
        }
        UIUtil.dispatchAllInvocationEvents()
    }
}
