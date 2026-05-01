package lang.ktav.lsp

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

/**
 * Pure-function tests for [KtavServerDiscovery.platformTriple]. No
 * IntelliJ platform required — exercises the OS/arch string mapping in
 * isolation.
 */
class PlatformTripleTest {

    @Test
    fun `linux amd64`() {
        assertEquals("linux-x64", KtavServerDiscovery.platformTriple("Linux", "amd64"))
    }

    @Test
    fun `linux x86_64`() {
        assertEquals("linux-x64", KtavServerDiscovery.platformTriple("Linux", "x86_64"))
    }

    @Test
    fun `linux aarch64`() {
        assertEquals("linux-arm64", KtavServerDiscovery.platformTriple("Linux", "aarch64"))
    }

    @Test
    fun `mac x86_64`() {
        assertEquals("darwin-x64", KtavServerDiscovery.platformTriple("Mac OS X", "x86_64"))
    }

    @Test
    fun `mac aarch64`() {
        assertEquals("darwin-arm64", KtavServerDiscovery.platformTriple("Mac OS X", "aarch64"))
    }

    @Test
    fun `windows 10 amd64`() {
        assertEquals("win32-x64", KtavServerDiscovery.platformTriple("Windows 10", "amd64"))
    }

    @Test
    fun `windows 11 aarch64`() {
        assertEquals("win32-arm64", KtavServerDiscovery.platformTriple("Windows 11", "aarch64"))
    }

    @Test
    fun `freebsd amd64 returns null`() {
        // Locked-in choice: unknown OS → null (fall through to PATH), not
        // silently treated as Linux. See kdoc on platformTriple.
        assertNull(KtavServerDiscovery.platformTriple("FreeBSD", "amd64"))
    }

    @Test
    fun `linux i386 returns null`() {
        assertNull(KtavServerDiscovery.platformTriple("Linux", "i386"))
    }

    @Test
    fun `case insensitive os name`() {
        assertEquals("linux-x64", KtavServerDiscovery.platformTriple("LINUX", "amd64"))
    }
}
