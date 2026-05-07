package lang.ktav

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectManagerListener

/**
 * Registers TextMate bundle on project open.
 * This handles both IDE startup and dynamic plugin loading.
 */
class KtavProjectActivity : ProjectManagerListener {
    override fun projectOpened(project: Project) {
        log.info("Ktav: projectOpened - registering TextMate bundle")
        KtavTextMateLoader.registerBundleIfNeeded()
    }

    companion object {
        private val log = Logger.getInstance(KtavProjectActivity::class.java)
    }
}
