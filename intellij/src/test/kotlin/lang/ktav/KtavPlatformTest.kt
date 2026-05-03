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
 * **STATUS — currently disabled.** On the gradle-intellij-platform 2.2
 * test fixture, the plugin's main `plugin.xml` extensions
 * (`fileType` / `lang.commenter`) are not registered into the
 * `BasePlatformTestCase` project, so `myFixture.file.fileType`
 * resolves to `PlainTextFileType` instead of [KtavFileType] and the
 * commenter action no-ops. The lightweight unit tests under
 * `KtavFileTypeTest`, `KtavLanguageTest`, `KtavCommenterTest`,
 * `lsp/PlatformTripleTest`, `lsp/KtavConfigurableLogicTest`, and
 * `lsp/KtavServerDiscoveryTest` still cover the equivalent invariants
 * in isolation. Tracking issue: figure out the correct
 * `intellijPlatform { testFramework(...) }` + `<idea-plugin>`
 * wiring so the manifest is loaded into the test sandbox; until
 * then keep the deeper smoke at `verifyPlugin` / real-IDE install.
 */
@Suppress("unused")
class KtavPlatformTest : BasePlatformTestCase() {

    @org.junit.Ignore("plugin.xml extensions not registered in test fixture; see kdoc")
    fun `test ktav file gets KtavFileType`() {
        myFixture.configureByText("a.ktav", "port: 8080\n")
        assertEquals(KtavFileType, myFixture.file.fileType)
    }

    @org.junit.Ignore("plugin.xml extensions not registered in test fixture; see kdoc")
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
