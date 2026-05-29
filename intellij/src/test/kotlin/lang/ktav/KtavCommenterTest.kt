package lang.ktav

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

class KtavCommenterTest {

    private val commenter = KtavCommenter()

    @Test
    fun `line comment prefix is double hash with trailing space`() {
        // Spec 0.5.0: comments use `##` (a single `#` is content).
        assertEquals("## ", commenter.lineCommentPrefix)
    }

    @Test
    fun `block comments are unsupported`() {
        assertNull(commenter.blockCommentPrefix)
        assertNull(commenter.blockCommentSuffix)
        assertNull(commentedBlockCommentPrefix())
        assertNull(commentedBlockCommentSuffix())
    }

    private fun commentedBlockCommentPrefix(): String? = commenter.commentedBlockCommentPrefix
    private fun commentedBlockCommentSuffix(): String? = commenter.commentedBlockCommentSuffix
}
