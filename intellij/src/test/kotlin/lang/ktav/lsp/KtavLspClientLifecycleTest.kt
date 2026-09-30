package lang.ktav.lsp

import com.google.gson.JsonObject
import com.intellij.testFramework.TestApplicationManager
import lang.ktav.lsp.client.LspTransport
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Test
import java.time.Duration
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class KtavLspClientLifecycleTest {
    companion object {
        @BeforeAll
        @JvmStatic
        fun initializeApplication() { TestApplicationManager.getInstance() }
    }

    @Test
    fun `close during process creation terminates the late child without waiting for initialize`() {
        TestLspProcess(silentInitialize = true).use { server ->
            val created = CountDownLatch(1)
            val release = CountDownLatch(1)
            val client = server.client {
                server.start().also {
                    created.countDown()
                    check(release.await(10, TimeUnit.SECONDS))
                }
            }
            val initialization = client.initialize()
            try {
                assertTrue(created.await(10, TimeUnit.SECONDS))
                assertTimeoutPreemptively(Duration.ofSeconds(5)) { client.close() }
                assertTrue(server.process.isAlive)
                release.countDown()
                initialization.get(10, TimeUnit.SECONDS)
                assertTrue(server.process.waitFor(5, TimeUnit.SECONDS))
                assertFalse(server.process.isAlive)
                assertFalse(java.nio.file.Files.exists(server.gate.resolve("initialize.received")))
                assertTrue(client.sendFormatting(JsonObject()).isCompletedExceptionally)
            } finally {
                release.countDown()
                client.close()
            }
        }
    }

    @Test
    fun `close while initialize is unanswered kills the child before closing blocked streams`() {
        TestLspProcess(silentInitialize = true).use { server ->
            val client = server.client()
            val initialization = client.initialize()
            try {
                server.awaitFile("initialize.received")
                assertTimeoutPreemptively(Duration.ofSeconds(5)) { client.close() }
                assertTrue(server.process.waitFor(5, TimeUnit.SECONDS))
                assertFalse(server.process.isAlive)
                assertTrue(client.sendFormatting(JsonObject()).isCompletedExceptionally)
            } finally {
                client.close()
            }
            // Transport request completion may be exceptional after close, but must not resurrect readiness.
            initialization.handle { _, _ -> null }.get(5, TimeUnit.SECONDS)
            assertTrue(client.sendFormatting(JsonObject()).isCompletedExceptionally)
        }
    }

    @Test
    fun `transport close fails existing requests and rejects a captured transport used after close`() {
        TestLspProcess(silentInitialize = true).use { server ->
            val transport = LspTransport(server.start()) {}
            val release = CountDownLatch(1)
            val captured = CountDownLatch(1)
            val lateRequest = java.util.concurrent.CompletableFuture.supplyAsync {
                val retainedTransport = transport
                captured.countDown()
                check(release.await(10, TimeUnit.SECONDS))
                retainedTransport.sendRequest("textDocument/formatting", JsonObject())
            }
            try {
                val pending = transport.sendRequest("initialize", JsonObject())
                server.awaitFile("initialize.received")
                assertTrue(captured.await(10, TimeUnit.SECONDS))
                transport.close()
                assertTrue(pending.isCompletedExceptionally)
                release.countDown()
                val rejected = lateRequest.get(5, TimeUnit.SECONDS)
                assertTrue(rejected.isCompletedExceptionally)
                assertFalse(java.nio.file.Files.exists(server.gate.resolve("format.received")))
            } finally {
                release.countDown()
                transport.close()
                lateRequest.get(5, TimeUnit.SECONDS)
            }
        }
    }
}
