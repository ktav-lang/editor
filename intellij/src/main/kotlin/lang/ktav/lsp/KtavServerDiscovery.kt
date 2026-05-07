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
 * Discovery order:
 *   1. Explicit override from [KtavSettings] (`serverPath`).
 *   2. Bundled binary inside the plugin distribution.
 *   3. Bare `ktav-lsp` — let the OS locate it via PATH.
 */
object KtavServerDiscovery {

    private val log = Logger.getInstance(KtavServerDiscovery::class.java)

    private const val PLUGIN_ID = "lang.ktav"
    private const val BINARY_BASENAME = "ktav-lsp"

    fun resolve(): List<String> {
        log.info("[Ktav Discovery] Starting binary resolution")
        return resolveWith { KtavSettings.getInstance().state.serverPath }
    }

    internal fun resolveWith(serverPathSupplier: () -> String): List<String> {
        val configured = serverPathSupplier()
        log.info("[Ktav Discovery] Configured path from settings: '$configured'")

        configuredPath(configured)?.let {
            log.info("[Ktav Discovery] Using configured path: $it")
            return listOf(it.toString())
        }

        bundledPath()?.let {
            log.info("[Ktav Discovery] Using bundled path: $it")
            return listOf(it.toString())
        }

        log.warn("[Ktav Discovery] No bundled binary found, falling back to PATH lookup: '$BINARY_BASENAME'")
        return listOf(BINARY_BASENAME)
    }

    private fun configuredPath(raw: String): Path? {
        val trimmed = raw.trim()
        if (trimmed.isEmpty()) {
            log.info("[Ktav Discovery] No configured path set")
            return null
        }
        val p = runCatching { Path.of(trimmed) }.getOrNull() ?: run {
            log.warn("[Ktav Discovery] Configured path invalid: '$trimmed'")
            return null
        }
        return if (Files.isRegularFile(p)) {
            log.info("[Ktav Discovery] Configured path is valid file: $p")
            p
        } else {
            log.warn("[Ktav Discovery] Configured path is not a regular file: $p")
            null
        }
    }

    private fun bundledPath(): Path? {
        log.info("[Ktav Discovery] Looking for plugin: $PLUGIN_ID")

        val plugin = runCatching {
            PluginManagerCore.getPlugin(PluginId.getId(PLUGIN_ID))
        }.getOrNull()

        if (plugin == null) {
            log.warn("[Ktav Discovery] Plugin not found via PluginManagerCore")
            return null
        }

        val root = plugin.pluginPath
        if (root == null) {
            log.warn("[Ktav Discovery] Plugin has no pluginPath")
            return null
        }
        log.info("[Ktav Discovery] Plugin root: $root")

        val osName = SystemInfo.OS_NAME
        val osArch = SystemInfo.OS_ARCH
        log.info("[Ktav Discovery] OS_NAME='$osName', OS_ARCH='$osArch'")

        val triple = platformTriple(osName, osArch)
        if (triple == null) {
            log.warn("[Ktav Discovery] Unsupported platform: $osName/$osArch")
            return null
        }
        log.info("[Ktav Discovery] Platform triple: $triple")

        val name = BINARY_BASENAME + exeSuffix()
        log.info("[Ktav Discovery] Binary name: $name")

        val candidates = listOf(
            root.resolve("bin").resolve(triple).resolve(name),
            root.resolve("lib").resolve("bin").resolve(triple).resolve(name),
        )

        for (c in candidates) {
            val exists = Files.isRegularFile(c)
            log.info("[Ktav Discovery] Candidate: $c → exists=$exists")
            if (exists) return c
        }

        log.warn("[Ktav Discovery] No bundled binary at any candidate path")
        return null
    }

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
