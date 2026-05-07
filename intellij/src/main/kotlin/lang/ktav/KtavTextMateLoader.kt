package lang.ktav

import com.intellij.ide.AppLifecycleListener
import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.extensions.PluginId
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectManagerListener
import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Registers the bundled Ktav TextMate grammar with IntelliJ's TextMate
 * plugin at IDE startup (via AppLifecycleListener) and on first project
 * open (via ProjectManagerListener).
 *
 * The TextMate engine's bundle-registration API (`TextMateService`,
 * `BundleType`, etc.) lives in the bundled `org.jetbrains.plugins.textmate`
 * plugin and is **internal** — its package and method names have shifted
 * between 2024.x and 2025.x builds. To register without breaking on
 * platform updates we look the API up via reflection and degrade
 * gracefully (WARN log + the user can still register the bundle by hand
 * via Settings → Editor → TextMate Bundles) if any step fails.
 */
class KtavTextMateLoader : AppLifecycleListener, ProjectManagerListener {

    private val log = Logger.getInstance(KtavTextMateLoader::class.java)
    private val registered = AtomicBoolean(false)

    override fun appFrameCreated(commandLineArgs: List<String>) {
        log.info("Ktav: appFrameCreated lifecycle hook fired")
        if (registered.compareAndSet(false, true)) {
            registerTextMateBundle()
        }
    }

    override fun projectOpened(project: Project) {
        log.info("Ktav: projectOpened lifecycle hook fired")
        if (registered.compareAndSet(false, true)) {
            registerTextMateBundle()
        }
    }

    private fun registerTextMateBundle() {
        if (!verifyBundleResources()) {
            log.warn(
                "Ktav TextMate bundle resources not found on the plugin " +
                    "classpath; syntax highlighting will be unavailable. " +
                    "Expected '$GRAMMAR_RESOURCE' and '$LANGUAGE_CONFIG_RESOURCE'.",
            )
            return
        }

        // Try file-system path first (for dev/unpacked mode)
        val bundlePathFs = resolveBundlePath()
        if (bundlePathFs != null) {
            log.info("Ktav: attempting to register TextMate bundle at $bundlePathFs")
            val ok = tryRegisterBundle(bundlePathFs)
            if (ok) {
                log.info("✓ Ktav TextMate bundle registered successfully (file-system mode)")
                return
            }
        }

        // Fallback: extract from JAR and register
        log.info("Ktav: file-system bundle not found; attempting JAR extraction mode")
        val ok = tryRegisterBundleFromJar()
        if (ok) {
            log.info("✓ Ktav TextMate bundle registered successfully (JAR mode)")
        } else {
            log.warn(
                "⚠ Ktav: TextMate auto-registration failed. Users can add the bundle " +
                    "manually via Settings → Editor → TextMate Bundles",
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
     * Extract TextMate bundle from plugin JAR and register it.
     * This works for packaged plugins where resources are inside JAR files.
     */
    private fun tryRegisterBundleFromJar(): Boolean {
        return try {
            val plugin = PluginManagerCore.getPlugin(PluginId.getId(PLUGIN_ID)) ?: return false
            val root = plugin.pluginPath ?: return false

            // Look for JAR file in lib/ (standard plugin layout)
            val libDir = root.resolve("lib")
            if (!Files.isDirectory(libDir)) {
                log.info("Ktav: lib/ directory not found at $libDir")
                return false
            }

            val jarFile = Files.list(libDir).use { stream ->
                stream.filter { it.fileName.toString().startsWith("ktav-intellij") && it.fileName.toString().endsWith(".jar") }
                    .findFirst()
                    .orElse(null)
            }

            if (jarFile == null) {
                log.warn("Ktav: No ktav-intellij jar found in $libDir")
                return false
            }

            log.info("Ktav: found plugin JAR at $jarFile")

            // Register JAR path as TextMate bundle
            val cl = KtavTextMateLoader::class.java.classLoader
            val serviceCls = runCatching {
                Class.forName("org.jetbrains.plugins.textmate.TextMateService", true, cl)
            }.getOrNull() ?: return false

            val getInstance = serviceCls.getMethod("getInstance")
            val service = getInstance.invoke(null) ?: return false

            val method = serviceCls.methods.firstOrNull {
                it.name == "registerEnabledBundle" && it.parameterCount in 1..2
            } ?: return false

            // Try to register the JAR file directly
            val bundleTypeCls = runCatching {
                Class.forName("org.jetbrains.plugins.textmate.bundles.BundleType", true, cl)
            }.getOrNull()

            val args: Array<Any?> = when (method.parameterCount) {
                1 -> arrayOf(jarFile)
                2 -> {
                    val second = bundleTypeCls?.let { runCatching {
                        @Suppress("UNCHECKED_CAST")
                        java.lang.Enum.valueOf(it as Class<out Enum<*>>, "TextMate")
                    }.getOrNull() }
                    arrayOf(jarFile, second)
                }
                else -> return false
            }

            method.invoke(service, *args)
            true
        } catch (t: Throwable) {
            log.warn("Ktav: TextMate JAR-based registration failed", t)
            false
        }
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
