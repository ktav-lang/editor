package lang.ktav

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/**
 * Verifies that the TextMate bundle resources synced into
 * `src/main/resources/grammars/ktav/` by the `syncGrammars` Gradle task
 * are actually on the test classpath.
 *
 * If this test fails, either:
 *  - `./gradlew syncGrammars` has not run yet (the test task depends on
 *    `processResources`, which depends on `syncGrammars`, so this is
 *    only possible from a partially configured build), or
 *  - the upstream `editor/grammars/ktav.tmLanguage.json` was renamed.
 */
class KtavTextMateLoaderTest {

    @Test
    fun `grammar json is on the classpath`() {
        val url = javaClass.classLoader.getResource(KtavTextMateLoader.GRAMMAR_RESOURCE)
        assertNotNull(url, "expected ${KtavTextMateLoader.GRAMMAR_RESOURCE} on the test classpath")
    }

    @Test
    fun `language configuration json is on the classpath`() {
        val url = javaClass.classLoader.getResource(KtavTextMateLoader.LANGUAGE_CONFIG_RESOURCE)
        assertNotNull(url, "expected ${KtavTextMateLoader.LANGUAGE_CONFIG_RESOURCE} on the test classpath")
    }

    @Test
    fun `bundle resources present probe returns true`() {
        assertTrue(
            KtavTextMateLoader.bundleResourcesPresent(),
            "both bundle resources should be discoverable on the classpath",
        )
    }

    @Test
    fun `grammar json declares ktav scope`() {
        val text = javaClass.classLoader
            .getResource(KtavTextMateLoader.GRAMMAR_RESOURCE)!!
            .readText()
        assertTrue(
            text.contains("\"scopeName\""),
            "tmLanguage.json must define a scopeName",
        )
        assertTrue(
            text.contains("ktav", ignoreCase = true),
            "tmLanguage.json should reference the ktav language",
        )
    }

    @Test
    fun `loader companion exposes stable resource paths`() {
        // Lock the path strings — plugin.xml and the syncGrammars task
        // both encode the same layout, so a rename here means a coordinated
        // change in build.gradle.kts.
        assertEquals(
            "grammars/ktav/Syntaxes/ktav.tmLanguage.json",
            KtavTextMateLoader.GRAMMAR_RESOURCE,
        )
        assertEquals(
            "grammars/ktav/language-configuration.json",
            KtavTextMateLoader.LANGUAGE_CONFIG_RESOURCE,
        )
    }
}
