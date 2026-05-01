package lang.ktav

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/**
 * Pure-Kotlin checks against the [KtavLanguage] singleton — no IntelliJ
 * sandbox required.
 */
class KtavLanguageTest {

    @Test
    fun `language id is ktav`() {
        assertEquals("ktav", KtavLanguage.id)
    }

    @Test
    fun `display name is human readable`() {
        assertEquals("Ktav", KtavLanguage.displayName)
    }

    @Test
    fun `language is case sensitive`() {
        // Ktav keys are case-sensitive per spec §3 — make sure the IDE
        // language definition agrees, otherwise references and rename
        // refactorings (when we add them) will fold case.
        assertTrue(KtavLanguage.isCaseSensitive)
    }

    @Test
    fun `file type points back at the language`() {
        assertEquals(KtavLanguage, KtavFileType.language)
    }
}
