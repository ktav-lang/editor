package lang.ktav.editor

import com.intellij.openapi.util.TextRange
import org.junit.Assert.assertEquals
import org.junit.Test

class KtavFoldingBuilderTest {
    @Test
    fun verbatim_body_brackets_are_opaque_and_single_hash_keys_open_folds() {
        assertEquals(listOf(TextRange(6, 13)), KtavFoldingBuilder.foldRanges("text: ((\n{\n))\n"))
        assertEquals(listOf(TextRange(8, 16)), KtavFoldingBuilder.foldRanges("#child: {\nx: 1\n}\n"))
        assertEquals(listOf(TextRange(8, 21)), KtavFoldingBuilder.foldRanges("#child: ((\na: true\n))\n"))
        assertEquals(listOf(TextRange(6, 20)), KtavFoldingBuilder.foldRanges("text: ((\n[\n## {\n)\n))\n"))
    }

    @Test
    fun raw_scalars_comments_and_pair_shaped_array_items_do_not_create_folds() {
        for (text in listOf(
            "raw:: {\na: 1\n",
            "raw:: ((\na: true\n",
            "## {\nx: 1\n",
            "hello\nname: (\ntrue\n",
            "hello\nname: {\ntrue\n",
        )) {
            assertEquals(text, emptyList<TextRange>(), KtavFoldingBuilder.foldRanges(text))
        }
        assertEquals(listOf(TextRange(7, 23)), KtavFoldingBuilder.foldRanges("items: [\nname: (\ntrue\n]\n"))
    }

    @Test
    fun all_spec_whitespace_and_line_terminators_preserve_nested_fold_ranges() {
        val horizontalWhitespace = listOf(
            '\u0009', '\u000B', '\u000C', '\u0020', '\u0085', '\u00A0', '\u1680',
            '\u2028', '\u2029', '\u202F', '\u205F', '\u3000',
        ) + ('\u2000'..'\u200A')
        for (ending in listOf("\n", "\r", "\r\n")) {
            for (ws in horizontalWhitespace) {
                val text = "#child: {$ws${ending}$ws" +
                    "items: [$ws${ending}$ws" +
                    "(($ws${ending}{${ending}$ws))$ws${ending}$ws]$ws${ending}$ws}$ws${ending}"
                assertEquals(
                    "U+${ws.code.toString(16)} with ${ending.map { it.code }}",
                    listOf(
                        TextRange(text.indexOf("(("), text.indexOf("))") + 2),
                        TextRange(text.indexOf('['), text.indexOf(']') + 1),
                        TextRange(text.indexOf('{'), text.lastIndexOf('}') + 1),
                    ),
                    KtavFoldingBuilder.foldRanges(text),
                )
            }
        }
    }

    @Test
    fun inline_structures_and_brackets_in_quoted_keys_do_not_steal_multiline_closers() {
        val text = "outer: {\n\"a{b\": [1, {v: true}]\nfirst \"name: 2\n}\n"
        assertEquals(listOf(TextRange(7, text.lastIndexOf('}') + 1)), KtavFoldingBuilder.foldRanges(text))
    }

    @Test
    fun non_spec_whitespace_does_not_turn_scalar_brackets_into_multiline_folds() {
        for (ws in listOf('\u180E', '\u200B', '\uFEFF')) {
            assertEquals(emptyList<TextRange>(), KtavFoldingBuilder.foldRanges("hello\n{$ws\ntrue\n}\n"))
            assertEquals(emptyList<TextRange>(), KtavFoldingBuilder.foldRanges("text: (($ws\ntrue\n))\n"))
        }
    }
}
