package lang.ktav

import com.intellij.lang.Language

/**
 * The IntelliJ [Language] handle for Ktav.
 *
 * Used as the discriminator key by the file type, the commenter, and any
 * future language-aware extension we register (formatter, structure view,
 * inspections, etc.). The string id `"ktav"` matches the `language="ktav"`
 * attribute on the `<fileType>`/`<lang.commenter>` extensions in
 * `plugin.xml` — keep them in sync.
 *
 * LSP integration status
 * ----------------------
 * The reference [`ktav-lsp`](../lsp) server already exists and exposes
 * diagnostics, hover, completion, document symbols and semantic tokens.
 * In-plugin LSP wiring (via the
 * [LSP4IJ](https://github.com/redhat-developer/lsp4ij) marketplace
 * plugin) is **deliberately deferred** to a follow-up release because:
 *
 *  - The new `org.jetbrains.intellij.platform` 2.x Gradle plugin
 *    requires a specific LSP4IJ version compatible with our IDE
 *    `sinceBuild = 243` / `untilBuild = 251.*` window. Pinning the
 *    wrong version turns into a class-load failure rather than a build
 *    error, so the version pair must be verified on a real IDE first.
 *  - Bundling or auto-discovering the `ktav-lsp` binary is explicitly
 *    out of scope for the current pass (per AGENTS guidance).
 *
 * Until then, users who want diagnostics can install LSP4IJ from the
 * marketplace and point it at a `ktav-lsp` binary on their PATH — see
 * the README for the exact steps.
 */
object KtavLanguage : Language("ktav") {
    override fun getDisplayName(): String = "Ktav"
    override fun isCaseSensitive(): Boolean = true
}
