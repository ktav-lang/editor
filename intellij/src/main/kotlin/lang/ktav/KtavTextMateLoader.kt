package lang.ktav

import com.intellij.ide.AppLifecycleListener
import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.extensions.PluginId
import java.nio.file.Files
import java.nio.file.Path

/**
 * Registers the bundled Ktav TextMate grammar with IntelliJ's TextMate
 * plugin at IDE startup.
 *
 * The TextMate engine's bundle-registration API (`TextMateService`,
 * `BundleType`, etc.) lives in the bundled `org.jetbrains.plugins.textmate`
 * plugin and is **internal** — its package and method names have shifted
 * between 2024.x and 2025.x builds. To register without breaking on
 * platform updates we look the API up via reflection and degrade
 * gracefully (WARN log + the user can still register the bundle by hand
 * via Settings → Editor → TextMate Bundles) if any step fails.
 */
class KtavTextMateLoader : AppLifecycleListener {

    private val log = Logger.getInstance(KtavTextMateLoader::class.java)

    override fun appFrameCreated(commandLineArgs: List<String>) {
        if (!verifyBundleResources()) {
            log.warn(
                "Ktav TextMate bundle resources not found on the plugin " +
                    "classpath; syntax highlighting will be unavailable. " +
                    "Expected '$GRAMMAR_RESOURCE' and '$LANGUAGE_CONFIG_RESOURCE'.",
            )
            return
        }

        val bundlePath = resolveBundlePath()
        if (bundlePath == null) {
            log.warn("Ktav: could not locate plugin bundle directory; TextMate auto-registration skipped.")
            return
        }

        val ok = tryRegisterBundle(bundlePath)
        if (ok) {
            log.info("Ktav TextMate bundle registered at $bundlePath")
        } else {
            log.warn(
                "Ktav: TextMate auto-registration failed (platform API not " +
                    "found via reflection). Users can still add the bundle " +
                    "manually via Settings -> Editor -> TextMate Bundles, " +
                    "pointing at: $bundlePath",
            )
        }
    }

    /** Resolve the on-disk directory containing this plugin's grammar bundle. */
    private fun resolveBundlePath(): Path? {
        val plugin = PluginManagerCore.getPlugin(PluginId.getId(PLUGIN_ID)) ?: return null
        val root = plugin.pluginPath ?: return null
        // Try the canonical layouts in order: `lib/grammars/ktav` (jar
        // unpack), then plain `grammars/ktav` (development run).
        val candidates = listOf(
            root.resolve("lib").resolve("grammars").resolve("ktav"),
            root.resolve("grammars").resolve("ktav"),
            root.resolve("classes").resolve("grammars").resolve("ktav"),
        )
        return candidates.firstOrNull { Files.isDirectory(it) }
    }

    /**
     * Reflection-based call to `TextMateService.registerEnabledBundle(path,
     * BundleType.TextMate)`. Returns true on success.
     */
    private fun tryRegisterBundle(bundlePath: Path): Boolean {
        return try {
            val cl = KtavTextMateLoader::class.java.classLoader
            val serviceCls = runCatching {
                Class.forName("org.jetbrains.plugins.textmate.TextMateService", true, cl)
            }.getOrNull() ?: return false

            val getInstance = serviceCls.getMethod("getInstance")
            val service = getInstance.invoke(null) ?: return false

            val bundleTypeCls = runCatching {
                Class.forName("org.jetbrains.plugins.textmate.bundles.BundleType", true, cl)
            }.getOrNull()

            // Find a `registerEnabledBundle` method irrespective of exact
            // signature drift.
            val method = serviceCls.methods.firstOrNull {
                it.name == "registerEnabledBundle" && it.parameterCount in 1..2
            } ?: return false

            val args: Array<Any?> = when (method.parameterCount) {
                1 -> arrayOf(bundlePath)
                2 -> {
                    val second = bundleTypeCls?.let { runCatching {
                        @Suppress("UNCHECKED_CAST")
                        java.lang.Enum.valueOf(it as Class<out Enum<*>>, "TextMate")
                    }.getOrNull() }
                    arrayOf(bundlePath, second)
                }
                else -> return false
            }
            method.invoke(service, *args)
            true
        } catch (t: Throwable) {
            log.warn("Ktav: TextMate registerEnabledBundle reflection failed", t)
            false
        }
    }

    private fun verifyBundleResources(): Boolean {
        val cl = KtavTextMateLoader::class.java.classLoader
        return cl.getResource(GRAMMAR_RESOURCE) != null &&
            cl.getResource(LANGUAGE_CONFIG_RESOURCE) != null
    }

    companion object {
        const val PLUGIN_ID: String = "lang.ktav"

        /** Path of the TextMate grammar inside the plugin classpath. */
        const val GRAMMAR_RESOURCE: String = "grammars/ktav/Syntaxes/ktav.tmLanguage.json"

        /** Path of the language-configuration JSON inside the plugin classpath. */
        const val LANGUAGE_CONFIG_RESOURCE: String = "grammars/ktav/language-configuration.json"

        @JvmStatic
        fun bundleResourcesPresent(): Boolean {
            val cl = KtavTextMateLoader::class.java.classLoader
            return cl.getResource(GRAMMAR_RESOURCE) != null &&
                cl.getResource(LANGUAGE_CONFIG_RESOURCE) != null
        }
    }
}
