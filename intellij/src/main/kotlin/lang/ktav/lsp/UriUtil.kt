package lang.ktav.lsp

import com.intellij.openapi.vfs.VirtualFile
import java.io.File

/**
 * Normalizes a URI to the canonical Java/LSP form: `file:///D:/path/file.ktav`
 *
 * IntelliJ's `VirtualFile.url` produces `file://D:/...` (two slashes) on Windows,
 * while the LSP server canonicalizes paths to `file:///D:/...` (three slashes).
 * Without normalization, diagnostics keyed by server URI won't match the URI
 * used when calling didOpen, and the annotator can't find them.
 *
 * Always use [normalize] both when sending URIs to the server and when looking
 * up diagnostics.
 */
object UriUtil {

    /**
     * Convert any URI form to canonical `file:///<path>` form.
     */
    fun normalize(uri: String): String {
        // Strip "file:" prefix and any leading slashes
        val withoutScheme = if (uri.startsWith("file:")) uri.substring(5) else uri
        val pathStripped = withoutScheme.trimStart('/')
        // Re-build canonical form
        return "file:///$pathStripped"
    }

    /**
     * Get canonical URI from VirtualFile.
     */
    fun fromVirtualFile(file: VirtualFile): String = normalize(file.url)

    /**
     * Get canonical URI from a File path.
     */
    fun fromFile(file: File): String = file.toURI().toString().let { normalize(it) }
}
