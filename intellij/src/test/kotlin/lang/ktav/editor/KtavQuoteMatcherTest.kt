package lang.ktav.editor

import lang.ktav.highlighting.KtavQuotedKeyAnnotator
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Pure pairing rules for quoted key segments — no IntelliJ runtime. */
class KtavQuoteMatcherTest {

    @Test
    fun caret_touching_either_quote_selects_both_like_a_matching_brace() {
        // `"ключ в кавычках"` as a KEY token starting at offset 10.
        val key = "\"ключ в кавычках\""
        val open = 10
        val close = open + key.length - 1
        for (caret in listOf(open, open + 1, close, close + 1)) {
            assertEquals("caret $caret", listOf(open, close), KtavQuoteMatcher.quotePairAt(open, key, caret))
        }
        assertNull(KtavQuoteMatcher.quotePairAt(open, key, open + 5))
    }

    @Test
    fun bare_keys_and_mismatched_delimiters_never_pair() {
        assertNull(KtavQuoteMatcher.quotePairAt(0, "plain", 0))
        assertNull(KtavQuoteMatcher.quotePairAt(0, "first \"name\"", 6))
        assertFalse(KtavQuotedKeyAnnotator.isQuotedSegment("\"abc'"))
        assertFalse(KtavQuotedKeyAnnotator.isQuotedSegment("\""))
        for (key in listOf("'x'", "`y`", "\"\"\"")) assertTrue(key, KtavQuotedKeyAnnotator.isQuotedSegment(key))
    }
}
