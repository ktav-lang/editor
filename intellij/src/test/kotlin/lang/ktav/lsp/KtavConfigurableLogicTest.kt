package lang.ktav.lsp

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/**
 * Pure-logic tests for [KtavConfigurable.isModified]. No Swing or
 * IntelliJ application services required.
 */
class KtavConfigurableLogicTest {

    @Test
    fun `empty text and empty saved are equal`() {
        assertFalse(KtavConfigurable.isModified("", ""))
    }

    @Test
    fun `whitespace text and empty saved are equal`() {
        assertFalse(KtavConfigurable.isModified("   ", ""))
        assertFalse(KtavConfigurable.isModified("\t \n", ""))
    }

    @Test
    fun `leading and trailing whitespace ignored`() {
        assertFalse(KtavConfigurable.isModified("  /usr/bin/ktav-lsp  ", "/usr/bin/ktav-lsp"))
    }

    @Test
    fun `different text is modified`() {
        assertTrue(KtavConfigurable.isModified("/a/ktav-lsp", "/b/ktav-lsp"))
    }

    @Test
    fun `text against empty saved is modified`() {
        assertTrue(KtavConfigurable.isModified("/usr/bin/ktav-lsp", ""))
    }
}
