package lang.ktav.lsp.lifecycle

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.ProjectActivity

/**
 * Runs after a project is opened.
 *
 * Initializes LSP client and sends didOpen for all already-opened .ktav files.
 * This is critical because FileEditorManagerListener only fires on NEW file
 * opens — files opened before the listener registration are missed.
 */
class KtavStartupActivity : ProjectActivity {

    private val log = Logger.getInstance(KtavStartupActivity::class.java)

    override suspend fun execute(project: Project) {
        log.info("[Ktav Startup] Project opened: ${project.name}, basePath=${project.basePath}")
        println(">>> [Ktav Startup] Project opened: ${project.name}")

        ApplicationManager.getApplication().invokeLater {
            if (project.isDisposed) return@invokeLater
            val editorManager = FileEditorManager.getInstance(project)
            val listener = FileOpenListener(project)
            editorManager.openFiles.filter { it.extension == "ktav" }
                .forEach { listener.fileOpened(editorManager, it) }
        }
    }
}
