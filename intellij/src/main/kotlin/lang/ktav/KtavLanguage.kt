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
 * The reference [`ktav-lsp`](../lsp) server exposes diagnostics, hover,
 * completion, document symbols, semantic tokens and formatting. The
 * plugin talks to it through its own built-in LSP client
 * ([`KtavLspProjectService`][lang.ktav.lsp.lifecycle.KtavLspProjectService] /
 * [`KtavStartupActivity`][lang.ktav.lsp.lifecycle.KtavStartupActivity]) —
 * no third-party LSP plugin (e.g. LSP4IJ) is required.
 *
 * [`KtavServerDiscovery`][lang.ktav.lsp.client.KtavServerDiscovery] finds
 * the `ktav-lsp` binary in order: the path configured in Settings ->
 * Tools -> Ktav, then the per-platform binary bundled in the plugin
 * distribution (`lib/bin/<os>-<arch>/ktav-lsp`), then `PATH`.
 */
object KtavLanguage : Language("ktav") {
    init {
        com.intellij.openapi.diagnostic.Logger.getInstance(KtavLanguage::class.java)
            .info("[Ktav Language] KtavLanguage object created (id='ktav')")
        println(">>> [Ktav Language] KtavLanguage object created")
    }

    override fun getDisplayName(): String = "Ktav"
    override fun isCaseSensitive(): Boolean = true
}
