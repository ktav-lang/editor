package lang.ktav

import com.intellij.codeInsight.generation.actions.CommentByLineCommentAction
import com.intellij.openapi.actionSystem.ex.ActionUtil
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.DataContext
import com.intellij.testFramework.fixtures.BasePlatformTestCase

/**
 * Heavyweight platform test — spins up an in-process IDE fixture and
 * verifies that:
 *  - a `.ktav` file gets the [KtavFileType] resolved automatically, and
 *  - the line-commenter action emits `# ` as the prefix (the wiring of
 *    [KtavCommenter] in `plugin.xml`).
 *
 * If this test fails to even start (e.g. offline Gradle build can't
 * download the platform), CI's `verifyPlugin` will catch the deeper
 * smoke at integration time. The lightweight unit tests still cover
 * the constants in isolation.
 */
class KtavPlatformTest : BasePlatformTestCase() {

    fun `test ktav file gets KtavFileType`() {
        myFixture.configureByText("a.ktav", "port: 8080\n")
        assertEquals(KtavFileType, myFixture.file.fileType)
    }

    fun `test commenter toggles with hash-space prefix`() {
        myFixture.configureByText("a.ktav", "<caret>port: 8080\n")
        val action = CommentByLineCommentAction()
        val event = AnActionEvent.createFromAnAction(
            action,
            null,
            "test",
            DataContext { dataId ->
                when (dataId) {
                    com.intellij.openapi.actionSystem.CommonDataKeys.EDITOR.name -> myFixture.editor
                    com.intellij.openapi.actionSystem.CommonDataKeys.PROJECT.name -> project
                    com.intellij.openapi.actionSystem.CommonDataKeys.PSI_FILE.name -> myFixture.file
                    else -> null
                }
            },
        )
        ActionUtil.performActionDumbAwareWithCallbacks(action, event)
        val text = myFixture.editor.document.text
        assertTrue(
            "expected commented line to start with '# ', got: $text",
            text.startsWith("# "),
        )
    }
}
