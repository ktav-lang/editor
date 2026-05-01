package lang.ktav.lsp

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.redhat.devtools.lsp4ij.LanguageServerFactory
import com.redhat.devtools.lsp4ij.server.ProcessStreamConnectionProvider
import com.redhat.devtools.lsp4ij.server.StreamConnectionProvider

/**
 * LSP4IJ entry point — only loaded when the optional LSP4IJ dependency
 * is present (see ktav-lsp4ij.xml). The main plugin therefore must not
 * reference this class from non-optional `plugin.xml` extensions.
 */
class KtavLanguageServerFactory : LanguageServerFactory {

    override fun createConnectionProvider(project: Project): StreamConnectionProvider {
        return KtavConnectionProvider(project)
    }
}

private class KtavConnectionProvider(private val project: Project) : ProcessStreamConnectionProvider() {

    private val log = Logger.getInstance(KtavConnectionProvider::class.java)

    init {
        val command = KtavServerDiscovery.resolve()
        super.setCommands(command)
    }

    override fun start() {
        val cmd = commands
        val first = cmd?.firstOrNull()
        // If discovery returned the bare "ktav-lsp" fallback and there is
        // no such executable on PATH, the underlying ProcessBuilder will
        // throw IOException. We surface a friendlier hint up front.
        if (first.isNullOrBlank()) {
            warnMissing()
            throw IllegalStateException("Ktav LSP server command is empty")
        }
        try {
            super.start()
        } catch (t: Throwable) {
            warnMissing()
            throw t
        }
    }

    private fun warnMissing() {
        val msg = "Ktav LSP server not found. Set the path in " +
            "Settings -> Tools -> Ktav, or run `cargo install ktav-lsp`."
        log.warn(msg)
        runCatching {
            NotificationGroupManager.getInstance()
                .getNotificationGroup("Ktav")
                .createNotification(msg, NotificationType.WARNING)
                .notify(project)
        }
    }
}
