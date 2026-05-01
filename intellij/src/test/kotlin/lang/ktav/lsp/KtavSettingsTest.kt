package lang.ktav.lsp

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

/**
 * Round-trip the persisted state object. The full XML serialization
 * round-trip lives inside the IntelliJ platform tests; here we just
 * verify that `loadState` copies fields into the existing instance
 * (per `XmlSerializerUtil.copyBean` semantics) and that defaults are
 * sane.
 */
class KtavSettingsTest {

    @Test
    fun `default state has empty server path`() {
        val state = KtavSettings.State()
        assertEquals("", state.serverPath)
    }

    @Test
    fun `state copy round-trip preserves server path`() {
        val settings = KtavSettings()
        val incoming = KtavSettings.State(serverPath = "/usr/local/bin/ktav-lsp")
        settings.loadState(incoming)
        assertEquals("/usr/local/bin/ktav-lsp", settings.state.serverPath)
    }

    @Test
    fun `mutating returned state persists in component`() {
        val settings = KtavSettings()
        settings.state.serverPath = "C:\\bin\\ktav-lsp.exe"
        assertEquals("C:\\bin\\ktav-lsp.exe", settings.state.serverPath)
    }
}
