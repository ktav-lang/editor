package lang.ktav.lsp.lifecycle

import com.intellij.openapi.Disposable
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.CommandProcessor
import com.intellij.openapi.editor.impl.DocumentImpl
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.testFramework.runInEdtAndWait
import java.lang.reflect.Proxy
import java.util.concurrent.CompletableFuture
import java.util.concurrent.CountDownLatch
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

internal fun DocumentImpl.edit(text: String) {
    runInEdtAndWait {
        CommandProcessor.getInstance().runUndoTransparentAction {
            ApplicationManager.getApplication().runWriteAction { replaceString(0, textLength, text) }
        }
    }
}

internal fun ChangeTracker.Snapshot.withCurrentOnEdt(action: () -> Unit): Boolean {
    var current = false
    runInEdtAndWait {
        ApplicationManager.getApplication().runWriteAction { current = withCurrent(action) }
    }
    return current
}

internal fun DocumentImpl.syncListenerCount(): Int {
    val getter = DocumentImpl::class.java.getDeclaredMethod("getListeners").apply { isAccessible = true }
    return (getter.invoke(this) as Array<*>).count { it is FileChangeListener }
}

internal class TestProject(parent: Disposable, name: String) {
    @Volatile private var disposed = false
    val project: Project = Proxy.newProxyInstance(
        Project::class.java.classLoader, arrayOf(Project::class.java)
    ) { proxy, method, args ->
        when (method.name) {
            "isDisposed" -> disposed
            "getName", "toString" -> name
            "hashCode" -> System.identityHashCode(proxy)
            "equals" -> proxy === args?.get(0)
            "dispose" -> { disposed = true; null }
            "getServiceIfCreated" -> null
            "getService" -> {
                check(args?.get(0) == ChangeTracker::class.java)
                tracker
            }
            else -> error("Unexpected Project call: ${method.name}")
        }
    } as Project
    val tracker = ChangeTracker(project)

    init {
        Disposer.register(parent, project)
        Disposer.register(project, tracker)
    }
}

internal data class SyncMessage(val kind: String, val uri: String, val version: Int?, val text: String?)

internal class RecordingSyncClient {
    val messages = mutableListOf<SyncMessage>()
    val documents = mutableMapOf<String, String>()
    var failOpen = false
    var failChange = false
    var failClose = false
    var active = true
    val connection = DocumentSyncConnection(
        this,
        { uri, version, text ->
            check(!failOpen) { "Open failed" }
            documents[uri] = text
            messages += SyncMessage("open", uri, version, text)
        },
        { uri, version, text ->
            check(!failChange) { "Change failed" }
            documents[uri] = text
            messages += SyncMessage("change", uri, version, text)
        },
        { uri ->
            check(!failClose) { "Close failed" }
            documents.remove(uri)
            messages += SyncMessage("close", uri, null, null)
        },
        { active }
    )
}

internal class RecordingLifecycleClient : AutoCloseable {
    val initialization = CompletableFuture<Void>()
    val started = CountDownLatch(1)
    val starts = AtomicInteger()
    val notifications = AtomicInteger()
    val closes = AtomicInteger()
    val liveResource = AtomicBoolean(true)
    var failClose = false

    fun initialize(): CompletableFuture<Void> {
        starts.incrementAndGet()
        started.countDown()
        return initialization
    }

    fun notifyInitialized() { notifications.incrementAndGet() }

    override fun close() {
        closes.incrementAndGet()
        liveResource.set(false)
        check(!failClose) { "Close failed" }
    }
}
