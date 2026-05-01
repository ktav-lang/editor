package lang.ktav.lsp

import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.extensions.PluginId
import com.intellij.openapi.util.SystemInfo
import java.nio.file.Files
import java.nio.file.Path

/**
 * Resolves the command line used to launch the `ktav-lsp` server.
 *
 * Discovery order (mirrors the VS Code extension):
 *   1. Explicit override from [KtavSettings] (`serverPath`), if non-blank
 *      and the file exists on disk.
 *   2. Bundled binary inside the plugin distribution under
 *      `bin/<platform>-<arch>/ktav-lsp[.exe]`.
 *   3. Bare `ktav-lsp` — let the OS locate it via PATH / `cargo install`.
 *
 * Returns a list suitable for `ProcessBuilder` / LSP4IJ's
 * `setCommands(...)`. Never returns null: case 3 is the always-present
 * fallback. Whether the resulting command actually launches is checked
 * by the caller.
 */
object KtavServerDiscovery {

    private val log = Logger.getInstance(KtavServerDiscovery::class.java)

    private const val PLUGIN_ID = "lang.ktav"
    private const val BINARY_BASENAME = "ktav-lsp"

    fun resolve(): List<String> = resolveWith { KtavSettings.getInstance().state.serverPath }

    /**
     * Test-friendly variant: the caller supplies the configured path
     * lookup, so unit tests can drive discovery without bootstrapping
     * the IntelliJ application service container.
     */
    internal fun resolveWith(serverPathSupplier: () -> String): List<String> {
        configuredPath(serverPathSupplier())?.let { return listOf(it.toString()) }
        bundledPath()?.let { return listOf(it.toString()) }
        return listOf(BINARY_BASENAME)
    }

    private fun configuredPath(raw: String): Path? {
        val trimmed = raw.trim()
        if (trimmed.isEmpty()) return null
        val p = runCatching { Path.of(trimmed) }.getOrNull() ?: return null
        return if (Files.isRegularFile(p)) p else null
    }

    private fun bundledPath(): Path? {
        val plugin = runCatching {
            PluginManagerCore.getPlugin(PluginId.getId(PLUGIN_ID))
        }.getOrNull() ?: return null
        val root = plugin.pluginPath ?: return null
        val triple = platformTriple(SystemInfo.OS_NAME, SystemInfo.OS_ARCH) ?: return null
        val name = BINARY_BASENAME + exeSuffix()
        val candidates = listOf(
            root.resolve("bin").resolve(triple).resolve(name),
            root.resolve("lib").resolve("bin").resolve(triple).resolve(name),
        )
        return candidates.firstOrNull { Files.isRegularFile(it) }.also {
            if (it == null) {
                log.debug("Ktav: no bundled ktav-lsp binary at ${candidates.joinToString()}")
            }
        }
    }

    /**
     * Compute a `linux-x64` / `darwin-arm64` / `win32-x64` style triple from
     * raw `os.name` / `os.arch` JVM property values.
     *
     * Pure function — no `SystemInfo` access — so it can be unit-tested
     * outside an IntelliJ sandbox. The mapping mirrors the VS Code
     * extension's `process.platform` + `process.arch` directory naming so
     * that a single bundled-binary tree (`bin/<triple>/`) serves both
     * editors.
     *
     * OS detection is substring-based and case-insensitive (matches
     * `SystemInfo` behaviour): any name containing `windows` → `win32`,
     * `mac`/`darwin` → `darwin`, `linux` → `linux`. Anything else → `null`
     * (we do not silently treat e.g. FreeBSD as Linux — better to fall
     * through to PATH lookup than launch a binary built for a different
     * libc).
     *
     * Arch detection accepts the common JVM spellings: `amd64`/`x86_64` →
     * `x64`, `aarch64`/`arm64` → `arm64`. Anything else (`i386`, `ppc64`,
     * `riscv64`, ...) → `null`.
     */
    internal fun platformTriple(osName: String, osArch: String): String? {
        val osLower = osName.lowercase()
        val os = when {
            osLower.contains("windows") -> "win32"
            osLower.contains("mac") || osLower.contains("darwin") -> "darwin"
            osLower.contains("linux") -> "linux"
            else -> return null
        }
        val arch = when (osArch.lowercase()) {
            "amd64", "x86_64" -> "x64"
            "aarch64", "arm64" -> "arm64"
            else -> return null
        }
        return "$os-$arch"
    }

    private fun exeSuffix(): String = if (SystemInfo.isWindows) ".exe" else ""
}
