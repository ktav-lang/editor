package lang.ktav

import com.intellij.lang.Commenter

/**
 * Powers the editor's "Comment Line" action (`Ctrl/Cmd+/`).
 *
 * Ktav has only line comments — `#` to end-of-line. There is no block
 * comment syntax in the spec, so the block-* methods all return null,
 * which the platform interprets as "feature unavailable".
 */
class KtavCommenter : Commenter {
    override fun getLineCommentPrefix(): String = "# "
    override fun getBlockCommentPrefix(): String? = null
    override fun getBlockCommentSuffix(): String? = null
    override fun getCommentedBlockCommentPrefix(): String? = null
    override fun getCommentedBlockCommentSuffix(): String? = null
}
