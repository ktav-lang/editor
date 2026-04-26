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
 */
object KtavLanguage : Language("ktav") {
    override fun getDisplayName(): String = "Ktav"
    override fun isCaseSensitive(): Boolean = true
}
