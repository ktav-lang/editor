package lang.ktav.lsp

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.nio.file.Files
import java.nio.file.Path

/**
 * Unit tests for the discovery chain — no IntelliJ sandbox required.
 *
 * `resolveWith(...)` lets us inject the configured-path supplier so we
 * don't have to spin up an ApplicationManager just to set a single
 * String. The bundled-binary lookup goes through
 * `PluginManagerCore.getPlugin(...)`, which returns null outside a
 * running platform, so it falls through to the bare-command fallback.
 */
class KtavServerDiscoveryTest {

    @Test
    fun `explicit path that exists wins`(@TempDir dir: Path) {
        val fake = Files.createFile(dir.resolve("ktav-lsp-fake.exe"))
        val cmd = KtavServerDiscovery.resolveWith { fake.toString() }
        assertEquals(listOf(fake.toString()), cmd)
    }

    @Test
    fun `blank explicit path falls through to bare command`() {
        val cmd = KtavServerDiscovery.resolveWith { "" }
        assertEquals(listOf("ktav-lsp"), cmd)
    }

    @Test
    fun `whitespace explicit path falls through to bare command`() {
        val cmd = KtavServerDiscovery.resolveWith { "   " }
        assertEquals(listOf("ktav-lsp"), cmd)
    }

    @Test
    fun `non-existent explicit path falls through to bare command`(@TempDir dir: Path) {
        val missing = dir.resolve("does-not-exist")
        val cmd = KtavServerDiscovery.resolveWith { missing.toString() }
        // Without a bundled binary or running platform, the fallback is
        // the bare command name resolved via PATH.
        assertEquals(listOf("ktav-lsp"), cmd)
    }

    @Test
    fun `result is never empty`() {
        val cmd = KtavServerDiscovery.resolveWith { "" }
        assertTrue(cmd.isNotEmpty())
    }

    @Test
    fun `directory path falls through to bare command`(@TempDir dir: Path) {
        // Settings field pointing at a directory (not a regular file) —
        // discovery must reject it via Files.isRegularFile() and fall
        // through to PATH lookup.
        val sub = Files.createDirectory(dir.resolve("not-a-file"))
        val cmd = KtavServerDiscovery.resolveWith { sub.toString() }
        assertEquals(listOf("ktav-lsp"), cmd)
    }

    @Test
    fun `whitespace around existing path is trimmed and used`(@TempDir dir: Path) {
        val fake = Files.createFile(dir.resolve("ktav-lsp-fake.exe"))
        val padded = "  ${fake}\t "
        val cmd = KtavServerDiscovery.resolveWith { padded }
        // The trimmed path resolves to the real file, so it wins.
        assertEquals(listOf(fake.toString()), cmd)
    }
}
