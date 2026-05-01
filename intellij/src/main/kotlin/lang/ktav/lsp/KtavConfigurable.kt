package lang.ktav.lsp

import com.intellij.openapi.fileChooser.FileChooserDescriptorFactory
import com.intellij.openapi.options.Configurable
import com.intellij.openapi.ui.TextFieldWithBrowseButton
import com.intellij.ui.components.JBLabel
import com.intellij.util.ui.FormBuilder
import javax.swing.JComponent
import javax.swing.JPanel

/**
 * Application-level Settings page under Tools -> Ktav.
 *
 * Single field: the absolute path to a `ktav-lsp` binary. The Ktav
 * plugin's LSP integration consults this value first, then falls back
 * to a bundled binary (none in the current pass), then to the bare
 * command name resolved via PATH.
 *
 * The page is registered unconditionally — even when LSP4IJ is not
 * installed — so users can configure the path ahead of installing
 * LSP4IJ.
 */
class KtavConfigurable : Configurable {

    private var pathField: TextFieldWithBrowseButton? = null
    private var rootPanel: JPanel? = null

    override fun getDisplayName(): String = "Ktav"

    override fun createComponent(): JComponent {
        val field = TextFieldWithBrowseButton().apply {
            addBrowseFolderListener(
                "Select ktav-lsp Binary",
                "Path to the Ktav language-server executable",
                null,
                FileChooserDescriptorFactory.createSingleFileDescriptor(),
            )
            text = KtavSettings.getInstance().state.serverPath
        }
        pathField = field

        val help = JBLabel(
            "<html>" +
                "Discovery order:<ol>" +
                "<li>This explicit path (if set and the file exists).</li>" +
                "<li>Bundled binary inside the plugin distribution.</li>" +
                "<li><code>ktav-lsp</code> on your <code>PATH</code> " +
                "(install with <code>cargo install ktav-lsp</code>).</li>" +
                "</ol>LSP features (diagnostics, hover, completion, " +
                "document symbols, semantic tokens) require the " +
                "<a href=\"https://plugins.jetbrains.com/plugin/23257-lsp4ij\">" +
                "LSP4IJ</a> plugin to be installed." +
                "</html>",
        )

        val panel = FormBuilder.createFormBuilder()
            .addLabeledComponent("Server path:", field)
            .addComponent(help)
            .addComponentFillVertically(JPanel(), 0)
            .panel
        rootPanel = panel
        return panel
    }

    override fun isModified(): Boolean {
        val saved = KtavSettings.getInstance().state.serverPath
        val edited = pathField?.text ?: return false
        return isModified(edited, saved)
    }

    companion object {
        /**
         * Pure modified-detection helper. Both inputs are trimmed before
         * comparison so leading/trailing whitespace round-trips don't
         * register as "dirty"; an all-whitespace text against an empty
         * saved value is treated as equal (we never persist whitespace
         * anyway — it's stripped on `apply` via the discovery layer).
         */
        @JvmStatic
        fun isModified(currentText: String, savedPath: String): Boolean =
            currentText.trim() != savedPath.trim()
    }

    override fun apply() {
        val edited = pathField?.text ?: return
        KtavSettings.getInstance().state.serverPath = edited
    }

    override fun reset() {
        pathField?.text = KtavSettings.getInstance().state.serverPath
    }

    override fun disposeUIResources() {
        pathField = null
        rootPanel = null
    }
}
