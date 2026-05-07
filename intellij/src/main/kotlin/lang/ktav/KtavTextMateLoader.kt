package lang.ktav

import com.intellij.ide.AppLifecycleListener
import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.extensions.PluginId
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectManagerListener
import java.io.BufferedInputStream
import java.io.BufferedOutputStream
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardOpenOption
import java.util.concurrent.atomic.AtomicBoolean
import java.util.zip.ZipEntry
import java.util.zip.ZipInputStream

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
        registerBundleIfNeeded()
    }

    override fun projectOpened(project: Project) {
        log.info("Ktav: projectOpened lifecycle hook fired")
        registerBundleIfNeeded()
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

        try {
            // Extract bundle from JAR
            val bundleDir = extractBundleToUserDirectory()
            if (bundleDir == null || !Files.isDirectory(bundleDir)) {
                log.warn("⚠ Ktav: TextMate bundle extraction failed")
                return
            }

            log.info("Ktav: extracted TextMate bundle to $bundleDir")

            // Try to register the bundle with TextMateService
            registerBundleWithTextMate(bundleDir)
        } catch (t: Throwable) {
            log.warn("Ktav: failed to setup TextMate bundle: ${t.message}", t)
        }
    }

    /**
     * Register the bundle with TextMateService.
     * Attempts to enable the bundle in TextMate settings.
     */
    private fun registerBundleWithTextMate(bundlePath: Path) {
        try {
            // Get the IDE config directory and textmate.xml path
            val plugin = PluginManagerCore.getPlugin(PluginId.getId(PLUGIN_ID)) ?: return
            val root = plugin.pluginPath ?: return

            log.info("Ktav: plugin root = $root")

            // Try to find textmate.xml by looking up the directory tree
            var current = root
            var found = false
            while (current != null && current.nameCount > 0) {
                val optionsDir = current.resolve("options")
                val textmateXmlPath = optionsDir.resolve("textmate.xml")
                log.info("Ktav: checking for textmate.xml at $textmateXmlPath")
                if (Files.exists(textmateXmlPath)) {
                    log.info("Ktav: found textmate.xml")
                    updateTextMateSettings(textmateXmlPath, bundlePath)
                    found = true
                    break
                }
                current = current.parent
            }

            if (!found) {
                log.info("Ktav: could not find textmate.xml, will rely on reloadEnabledBundles")
            }

            // Try calling reloadEnabledBundles
            val cl = KtavTextMateLoader::class.java.classLoader
            val serviceCls = Class.forName("org.jetbrains.plugins.textmate.TextMateService", true, cl)
            val getInstance = serviceCls.getMethod("getInstance")
            val service = getInstance.invoke(null)

            if (service != null) {
                val reloadMethod = serviceCls.getMethod("reloadEnabledBundles")
                reloadMethod.invoke(service)
                log.info("✓ Ktav: called reloadEnabledBundles()")
            }
        } catch (t: Throwable) {
            log.warn("Ktav: failed to register TextMate bundle: ${t.message}")
            // This is not a fatal error - users can still register manually
        }
    }

    /**
     * Update the TextMate settings XML to enable the Ktav bundle.
     * This adds the bundle path to TextMateUserBundlesSettings.
     * Note: The JSON format expected by TextMateUserBundlesSettings may vary by IDE version.
     */
    private fun updateTextMateSettings(textmateXmlPath: Path, bundlePath: Path) {
        try {
            val content = String(Files.readAllBytes(textmateXmlPath), Charsets.UTF_8)

            // Parse the JSON from CDATA
            val match = Regex("<component name=\"TextMateUserBundlesSettings\"><!\\[CDATA\\[(.*)\\]\\]></component>")
                .find(content)

            if (match != null) {
                val jsonStr = match.groupValues[1]
                // Try multiple JSON formats since the expected format varies by IDE version
                // Format 1: Try as array of bundle entries (most likely for newer versions)
                val newJson = try {
                    // First try to parse as JSON to understand structure
                    if (jsonStr.trim() == "{}" || jsonStr.trim().isEmpty()) {
                        // Empty settings - try array format: [{"name":"ktav","enabled":true,"path":"..."}]
                        "[{\"name\":\"ktav\",\"enabled\":true,\"path\":\"$bundlePath\"}]"
                    } else {
                        // If it already has content, try appending to it
                        jsonStr.trimEnd().dropLast(1) + ",{\"name\":\"ktav\",\"enabled\":true,\"path\":\"$bundlePath\"}]"
                    }
                } catch (e: Exception) {
                    // Fallback to simple map format
                    "{\"ktav\": \"$bundlePath\"}"
                }

                val newContent = content.replace(match.value,
                    "<component name=\"TextMateUserBundlesSettings\"><![CDATA[$newJson]]></component>")

                Files.write(textmateXmlPath, newContent.toByteArray(Charsets.UTF_8))
                log.info("Ktav: updated TextMate settings with JSON: $newJson")
            }
        } catch (t: Throwable) {
            log.warn("Ktav: failed to update TextMate settings: ${t.message}")
        }
    }

    /**
     * Extract the Ktav bundle to a temporary directory.
     * Returns the path to the extracted bundle (.tmbundle), or null if extraction failed.
     */
    private fun extractBundleToUserDirectory(): Path? {
        return try {
            val plugin = PluginManagerCore.getPlugin(PluginId.getId(PLUGIN_ID)) ?: return null
            val root = plugin.pluginPath ?: return null

            // Try file-system path first (for dev/unpacked mode)
            val bundlePathFs = resolveBundlePath()
            if (bundlePathFs != null && Files.isDirectory(bundlePathFs)) {
                log.info("Ktav: using file-system bundle at $bundlePathFs")
                return bundlePathFs
            }

            // Extract from JAR
            val libDir = root.resolve("lib")
            val jarFile = Files.list(libDir).use { stream ->
                stream.filter {
                    val name = it.fileName.toString()
                    name.startsWith("ktav-intellij") &&
                    name.endsWith(".jar") &&
                    !name.contains("searchableOptions")
                }
                    .findFirst()
                    .orElse(null)
            } ?: return null

            log.info("Ktav: found plugin JAR at $jarFile")

            // Create a .tmbundle directory
            val bundleDir = Files.createTempDirectory("Ktav.tmbundle")
            bundleDir.toFile().deleteOnExit()

            extractBundleFromJarFlat(jarFile, bundleDir)

            if (Files.isDirectory(bundleDir) && Files.list(bundleDir).findAny().isPresent) {
                bundleDir
            } else {
                null
            }
        } catch (t: Throwable) {
            log.warn("Ktav: failed to extract bundle", t)
            null
        }
    }

    /** Extract the grammars/ktav directory from a JAR file, flattening the directory structure. */
    private fun extractBundleFromJarFlat(jarFile: Path, targetBundle: Path) {
        val bundlePrefix = "grammars/ktav/"
        ZipInputStream(BufferedInputStream(Files.newInputStream(jarFile))).use { zis ->
            var entry: ZipEntry? = zis.nextEntry
            while (entry != null) {
                if (entry.name.startsWith(bundlePrefix)) {
                    val relativePath = entry.name.substring(bundlePrefix.length)
                    if (relativePath.isNotEmpty()) {
                        val targetPath = targetBundle.resolve(relativePath)
                        if (entry.isDirectory) {
                            Files.createDirectories(targetPath)
                        } else {
                            Files.createDirectories(targetPath.parent)
                            BufferedOutputStream(Files.newOutputStream(targetPath, StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING)).use { fos ->
                                zis.copyTo(fos, 8192)
                            }
                        }
                    }
                }
                entry = zis.nextEntry
            }
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

    /** Extract the grammars/ktav directory from a JAR file. */
    private fun extractBundleFromJar(jarFile: Path, targetDir: Path) {
        val bundlePrefix = "grammars/ktav/"
        ZipInputStream(BufferedInputStream(Files.newInputStream(jarFile))).use { zis ->
            var entry: ZipEntry? = zis.nextEntry
            while (entry != null) {
                if (entry.name.startsWith(bundlePrefix)) {
                    val relativePath = entry.name.substring(bundlePrefix.length)
                    if (relativePath.isNotEmpty()) {
                        val targetPath = targetDir.resolve(entry.name)
                        if (entry.isDirectory) {
                            Files.createDirectories(targetPath)
                        } else {
                            Files.createDirectories(targetPath.parent)
                            BufferedOutputStream(Files.newOutputStream(targetPath, StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING)).use { fos ->
                                zis.copyTo(fos, 8192)
                            }
                        }
                    }
                }
                entry = zis.nextEntry
            }
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

        private val log = Logger.getInstance(KtavTextMateLoader::class.java)
        private val registered = AtomicBoolean(false)

        @JvmStatic
        fun registerBundleIfNeeded() {
            if (registered.compareAndSet(false, true)) {
                log.info("Ktav: registering TextMate bundle")
                try {
                    val loader = KtavTextMateLoader()
                    loader.registerTextMateBundle()
                } catch (e: Exception) {
                    log.warn("Ktav: failed to register TextMate bundle", e)
                }
            }
        }

        @JvmStatic
        fun bundleResourcesPresent(): Boolean {
            val cl = KtavTextMateLoader::class.java.classLoader
            return cl.getResource(GRAMMAR_RESOURCE) != null &&
                cl.getResource(LANGUAGE_CONFIG_RESOURCE) != null
        }
    }
}
