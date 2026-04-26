package lang.ktav

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/**
 * Smoke tests for the file-type registration metadata.
 *
 * These are pure-Kotlin checks against the singleton — they do not boot
 * the IntelliJ Platform sandbox, so they run fast and stay green even
 * without a configured IDE under test.
 */
class KtavFileTypeTest {

    @Test
    fun `file type has correct extension`() {
        assertEquals("ktav", KtavFileType.defaultExtension)
    }

    @Test
    fun `file type has correct name`() {
        assertEquals("Ktav", KtavFileType.name)
    }

    @Test
    fun `file type description mentions ktav`() {
        assertTrue(
            KtavFileType.description.contains("Ktav", ignoreCase = true),
            "description should mention the language name",
        )
    }

    @Test
    fun `language id matches plugin xml`() {
        assertEquals("ktav", KtavLanguage.id)
    }
}
