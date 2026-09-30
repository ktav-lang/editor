package lang.ktav.lsp

import com.google.gson.JsonObject
import com.intellij.openapi.application.PathManager
import lang.ktav.lsp.client.KtavLspClient
import java.io.File
import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit

internal class TestLspProcess(private val silentInitialize: Boolean = false) : AutoCloseable {
    val gate: Path = Files.createTempDirectory("ktav-lsp-lifecycle-")
    lateinit var process: Process
        private set
    private val creation = CompletableFuture<Process>()

    fun start(): Process {
        try {
            val classpath = listOf(TestLspServer::class.java, JsonObject::class.java).joinToString(File.pathSeparator) {
                checkNotNull(PathManager.getJarPathForClass(it)) { "No resource root for ${it.name}" }
            }
            process = ProcessBuilder(
                Path.of(System.getProperty("java.home"), "bin", "java").toString(),
                "-cp", classpath, TestLspServer::class.java.name, gate.toString(), silentInitialize.toString()
            ).redirectError(ProcessBuilder.Redirect.INHERIT).start()
            creation.complete(process)
            return process
        } catch (ex: Exception) {
            creation.completeExceptionally(ex)
            throw ex
        }
    }

    fun client(start: () -> Process = ::start) = KtavLspClient("test-stdio-child", gate.toString(), {}, {}, start)

    fun awaitFile(name: String): String {
        val child = creation.get(10, TimeUnit.SECONDS)
        val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(10)
        val path = gate.resolve(name)
        while (!Files.exists(path)) {
            check(child.isAlive) { "Protocol child exited with ${child.exitValue()} before $name" }
            check(System.nanoTime() < deadline) { "Timed out waiting for $name" }
            Thread.sleep(10)
        }
        return Files.readString(path)
    }

    fun awaitText(name: String, expected: String): String {
        val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(10)
        while (true) {
            val text = awaitFile(name)
            if (text == expected) return text
            check(System.nanoTime() < deadline) { "Timed out waiting for $name to contain $expected (got $text)" }
            Thread.sleep(10)
        }
    }

    fun releaseFormatting() { Files.writeString(gate.resolve("format.release"), "release") }

    override fun close() {
        if (::process.isInitialized) {
            process.destroyForcibly()
            check(process.waitFor(10, TimeUnit.SECONDS)) { "Protocol child did not stop" }
        }
        Files.list(gate).use { paths -> paths.forEach { Files.deleteIfExists(it) } }
        Files.deleteIfExists(gate)
    }
}
