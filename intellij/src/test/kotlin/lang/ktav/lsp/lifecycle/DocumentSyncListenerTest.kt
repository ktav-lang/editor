package lang.ktav.lsp.lifecycle

import com.intellij.openapi.editor.impl.DocumentImpl
import com.intellij.openapi.util.Disposer
import com.intellij.testFramework.TestApplicationManager
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource
import java.time.Duration
import java.util.concurrent.CompletableFuture
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class DocumentSyncListenerTest {
    companion object {
        @BeforeAll
        @JvmStatic
        fun initializeApplication() { TestApplicationManager.getInstance() }
    }

    private val root = Disposer.newDisposable()
    private val uri = "file:///shared.ktav"

    @AfterEach
    fun cleanup() = Disposer.dispose(root)

    @ParameterizedTest
    @ValueSource(booleans = [false, true])
    fun `distinct documents only update their owner regardless of attachment order`(reverse: Boolean) {
        val owners = listOf(TestProject(root, "A"), TestProject(root, "B"))
        val clients = listOf(RecordingSyncClient(), RecordingSyncClient())
        val documents = listOf(DocumentImpl("a: 1"), DocumentImpl("b: 1"))
        val uris = listOf("file:///a.ktav", "file:///b.ktav")
        val order = if (reverse) listOf(1, 0) else listOf(0, 1)
        order.forEach { owners[it].tracker.attach(uris[it], documents[it], clients[it].connection) }

        documents[1].edit("b: 2")
        documents[0].edit("a: 2")
        for (i in 0..1) {
            assertEquals(listOf("open", "change"), clients[i].messages.map { it.kind })
            assertEquals(listOf(1, 2), clients[i].messages.map { it.version })
            assertTrue(clients[i].messages.all { it.uri == uris[i] })
            assertEquals(documents[i].text, clients[i].documents[uris[i]])
        }
    }

    @ParameterizedTest
    @ValueSource(booleans = [false, true])
    fun `shared document keeps independent versions and closing one owner preserves the other`(reverse: Boolean) {
        val owners = listOf(TestProject(root, "A"), TestProject(root, "B"))
        val clients = listOf(RecordingSyncClient(), RecordingSyncClient())
        val document = DocumentImpl("v: 1")
        val first = if (reverse) 1 else 0
        val second = 1 - first
        owners[first].tracker.attach(uri, document, clients[first].connection)
        document.edit("v: 2")
        owners[second].tracker.attach(uri, document, clients[second].connection)
        assertEquals(2, document.syncListenerCount())
        document.edit("v: 3")
        assertEquals(listOf(1, 2, 3), clients[first].messages.map { it.version })
        assertEquals(listOf(1, 2), clients[second].messages.map { it.version })
        assertEquals("v: 2", clients[second].messages.first().text)

        owners[first].tracker.detach(uri)
        assertEquals(1, document.syncListenerCount())
        document.edit("v: 4")
        assertEquals("close", clients[first].messages.last().kind)
        assertEquals(3, clients[second].messages.last().version)
        assertEquals("v: 4", clients[second].documents[uri])

        owners[first].tracker.attach(uri, document, clients[first].connection)
        assertEquals(2, document.syncListenerCount())
        document.edit("v: 5")
        assertEquals(5, clients[first].messages.last().version)
        assertEquals(4, clients[second].messages.last().version)
        assertEquals("v: 5", clients[first].documents[uri])
        assertEquals("v: 5", clients[second].documents[uri])
    }

    @Test
    fun `repeated subscriptions deduplicate without resetting the server cache`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v: 1")
        owner.tracker.attach(uri, document, client.connection)
        document.edit("v: 2")
        owner.tracker.attach(uri, document, client.connection)
        document.edit("v: 3")
        assertEquals(1, document.syncListenerCount())
        assertEquals(listOf("open", "change", "change"), client.messages.map { it.kind })
        assertEquals(listOf(1, 2, 3), client.messages.map { it.version })
    }

    @ParameterizedTest
    @ValueSource(booleans = [false, true])
    fun `disposing either project removes only its listeners and prevents late attachment`(reverse: Boolean) {
        val owners = listOf(TestProject(root, "A"), TestProject(root, "B"))
        val clients = listOf(RecordingSyncClient(), RecordingSyncClient())
        val document = DocumentImpl("v: 1")
        val privateDocument = DocumentImpl("private: 1")
        for (i in 0..1) owners[i].tracker.attach(uri, document, clients[i].connection)
        val closed = if (reverse) 1 else 0
        val survivor = 1 - closed
        owners[closed].tracker.attach("file:///private.ktav", privateDocument, clients[closed].connection)
        val snapshot = owners[closed].tracker.snapshot(uri, document, clients[closed], document.text)!!

        Disposer.dispose(owners[closed].project)
        assertEquals(1, document.syncListenerCount())
        assertEquals(0, privateDocument.syncListenerCount())
        assertFalse(snapshot.isCurrent())
        assertTrue(clients[closed].documents.isEmpty())
        val count = clients[closed].messages.size
        owners[closed].tracker.attach(uri, document, clients[closed].connection)
        document.edit("v: 2")
        privateDocument.edit("private: 2")
        assertEquals(count, clients[closed].messages.size)
        assertEquals("v: 2", clients[survivor].documents[uri])
        assertNull(owners[closed].tracker.snapshot(uri, document, clients[closed], document.text))

        Disposer.dispose(owners[survivor].project)
        assertEquals(0, document.syncListenerCount())
        val survivorCount = clients[survivor].messages.size
        document.edit("v: 3")
        assertEquals(survivorCount, clients[survivor].messages.size)
    }

    @Test
    fun `failed close still removes the listener`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v: 1")
        owner.tracker.attach(uri, document, client.connection)
        client.failClose = true
        owner.tracker.detach(uri)
        assertEquals(0, document.syncListenerCount())
        document.edit("v: 2")
        assertEquals(1, client.messages.size)
        assertNull(owner.tracker.snapshot(uri, document, client, document.text))
    }

    @Test
    fun `failed open rolls back the listener and can retry`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient().apply { failOpen = true }
        val document = DocumentImpl("v: 1")
        owner.tracker.attach(uri, document, client.connection)
        document.edit("v: 2")
        assertTrue(client.messages.isEmpty())
        assertEquals(0, document.syncListenerCount())
        client.failOpen = false
        owner.tracker.attach(uri, document, client.connection)
        document.edit("v: 3")
        assertEquals(listOf(2, 3), client.messages.map { it.version })
        assertEquals("v: 2", client.messages.first().text)
    }

    @Test
    fun `unsent changes cannot supply a formatting snapshot but full sync recovers`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v: 1")
        owner.tracker.attach(uri, document, client.connection)
        client.failChange = true
        document.edit("v: 2")
        assertNull(owner.tracker.snapshot(uri, document, client, document.text))
        client.failChange = false
        document.edit("v: 3")
        assertTrue(owner.tracker.snapshot(uri, document, client, document.text)!!.isCurrent())
        assertEquals(3, client.messages.last().version)
        assertEquals("v: 3", client.documents[uri])
    }

    @Test
    fun `snapshot checks client document text and session identity including close reopen`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v: 1")
        owner.tracker.attach(uri, document, client.connection)
        assertNull(owner.tracker.snapshot(uri, document, RecordingSyncClient(), document.text))
        assertNull(owner.tracker.snapshot(uri, DocumentImpl(document.text), client, document.text))
        assertNull(owner.tracker.snapshot(uri, document, client, "v: stale"))
        val snapshot = owner.tracker.snapshot(uri, document, client, document.text)!!
        owner.tracker.detach(uri)
        owner.tracker.attach(uri, document, client.connection)
        assertFalse(snapshot.isCurrent())

        val reopened = owner.tracker.snapshot(uri, document, client, document.text)!!
        val replacementClient = RecordingSyncClient()
        owner.tracker.attach(uri, document, replacementClient.connection)
        assertFalse(reopened.isCurrent())
        document.edit("v: 2")
        assertEquals("close", client.messages.last().kind)
        assertEquals(listOf(1, 2), replacementClient.messages.map { it.version })
    }

    @Test
    fun `editing back to the same text and stamp still invalidates the old version`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v: 1")
        owner.tracker.attach(uri, document, client.connection)
        val stamp = document.modificationStamp
        val snapshot = owner.tracker.snapshot(uri, document, client, document.text)!!
        document.edit("v: 2")
        document.edit("v: 1")
        document.setModificationStamp(stamp)
        assertFalse(snapshot.isCurrent())
    }

    @Test
    fun `inactive client rejects changes snapshots and late attachments`() {
        val owner = TestProject(root, "owner")
        val client = RecordingSyncClient()
        val document = DocumentImpl("v: 1")
        owner.tracker.attach(uri, document, client.connection)
        val snapshot = owner.tracker.snapshot(uri, document, client, document.text)!!
        client.active = false
        assertFalse(snapshot.isCurrent())
        document.edit("v: 2")
        assertEquals(listOf("open"), client.messages.map { it.kind })
        assertNull(owner.tracker.snapshot(uri, document, client, document.text))
        owner.tracker.detach(uri)
        owner.tracker.attach(uri, document, client.connection)
        assertEquals(0, document.syncListenerCount())
    }

    @Test
    fun `project lifecycle rejects and releases a client created during close`() {
        val client = RecordingLifecycleClient()
        val entered = CountDownLatch(1)
        val release = CountDownLatch(1)
        val lifecycle = ProjectClientLifecycle(
            { false }, {
                entered.countDown()
                check(release.await(5, TimeUnit.SECONDS))
                client
            }, RecordingLifecycleClient::initialize, RecordingLifecycleClient::notifyInitialized
        ) { throw AssertionError(it) }
        val initialization = CompletableFuture.runAsync { lifecycle.ensureInitialized() }
        try {
            assertTrue(entered.await(5, TimeUnit.SECONDS))
            assertTimeoutPreemptively(Duration.ofSeconds(5)) { lifecycle.close() }
        } finally {
            release.countDown()
            initialization.get(5, TimeUnit.SECONDS)
            lifecycle.close()
        }
        assertEquals(0, client.starts.get())
        assertEquals(0, client.notifications.get())
        assertEquals(1, client.closes.get())
        assertFalse(client.liveResource.get())
        assertNull(lifecycle.getClient())
    }

    @Test
    fun `project disposal between construction and publication releases the candidate`() {
        val client = RecordingLifecycleClient()
        var disposed = false
        val lifecycle = ProjectClientLifecycle(
            { disposed }, { disposed = true; client },
            RecordingLifecycleClient::initialize, RecordingLifecycleClient::notifyInitialized
        ) { throw AssertionError(it) }
        lifecycle.ensureInitialized()
        assertEquals(0, client.starts.get())
        assertEquals(1, client.closes.get())
        assertFalse(client.liveResource.get())
        assertNull(lifecycle.getClient())
        lifecycle.close()
    }

    @Test
    fun `close during asynchronous startup cleans resources created after close without publishing`() {
        val client = RecordingLifecycleClient()
        val lifecycle = lifecycle(client)
        val initialization = CompletableFuture.runAsync { lifecycle.ensureInitialized() }
        try {
            assertTrue(client.started.await(5, TimeUnit.SECONDS))
            assertNull(lifecycle.getClient())
            assertTimeoutPreemptively(Duration.ofSeconds(5)) { lifecycle.close() }
            assertEquals(1, client.closes.get())
            client.liveResource.set(true)
            client.initialization.complete(null)
            initialization.get(5, TimeUnit.SECONDS)
            assertEquals(2, client.closes.get())
            assertFalse(client.liveResource.get())
            assertEquals(0, client.notifications.get())
            assertNull(lifecycle.getClient())
            lifecycle.ensureInitialized()
            assertEquals(1, client.starts.get())
        } finally {
            lifecycle.close()
            client.initialization.complete(null)
            initialization.get(5, TimeUnit.SECONDS)
        }
    }

    @ParameterizedTest
    @ValueSource(booleans = [false, true])
    fun `timeout preserves the raw completion trigger and cleans late startup success or failure`(fails: Boolean) {
        val client = RecordingLifecycleClient()
        val errors = mutableListOf<Exception>()
        val lifecycle = lifecycle(client, timeoutMillis = 0, onFailure = { errors += it })
        lifecycle.ensureInitialized()
        assertEquals(1, errors.size)
        assertEquals(1, client.closes.get())
        assertFalse(client.initialization.isDone)
        client.liveResource.set(true)
        if (fails) client.initialization.completeExceptionally(IllegalStateException("Late failure"))
        else client.initialization.complete(null)
        assertEquals(2, client.closes.get())
        assertFalse(client.liveResource.get())
        assertEquals(0, client.notifications.get())
        assertNull(lifecycle.getClient())
        lifecycle.ensureInitialized()
        assertEquals(1, client.starts.get())
        lifecycle.close()
        assertEquals(2, client.closes.get())
    }

    @Test
    fun `ready client is published only after notification and reused then closed once`() {
        val client = RecordingLifecycleClient().apply { initialization.complete(null) }
        var constructions = 0
        lateinit var lifecycle: ProjectClientLifecycle<RecordingLifecycleClient>
        lifecycle = ProjectClientLifecycle(
            { false }, { constructions++; client }, RecordingLifecycleClient::initialize,
            { assertNull(lifecycle.getClient()); it.notifyInitialized() }
        ) { throw AssertionError(it) }
        try {
            assertFalse(lifecycle.withClient(client) { error("Not initialized") })
            lifecycle.ensureInitialized()
            assertSame(client, lifecycle.getClient())
            assertFalse(lifecycle.withClient(RecordingLifecycleClient()) { error("Wrong client") })
            var delivered = false
            assertTrue(lifecycle.withClient(client) { delivered = true })
            assertTrue(delivered)
            lifecycle.ensureInitialized()
            assertEquals(1, constructions)
            assertEquals(1, client.starts.get())
            assertEquals(1, client.notifications.get())
        } finally {
            lifecycle.close()
        }
        lifecycle.close()
        lifecycle.ensureInitialized()
        assertNull(lifecycle.getClient())
        assertFalse(lifecycle.withClient(client) { error("Closed client") })
        assertEquals(1, constructions)
        assertEquals(1, client.closes.get())
    }

    @ParameterizedTest
    @ValueSource(strings = ["construct", "start", "future", "notify", "close"])
    fun `lifecycle failures release ownership and never expose a partially initialized client`(stage: String) {
        val client = RecordingLifecycleClient()
        if (stage == "future") client.initialization.completeExceptionally(IllegalStateException("Init failed"))
        else client.initialization.complete(null)
        client.failClose = stage == "close"
        val errors = mutableListOf<Exception>()
        var constructions = 0
        val lifecycle = ProjectClientLifecycle(
            { false }, {
                constructions++
                check(stage != "construct") { "Construction failed" }
                client
            }, {
                check(stage != "start") { "Start failed" }
                it.initialize()
            }, {
                check(stage != "notify") { "Notification failed" }
                it.notifyInitialized()
            }
        ) { errors += it }
        lifecycle.ensureInitialized()
        if (stage == "close") {
            assertSame(client, lifecycle.getClient())
            lifecycle.close()
        }
        assertNull(lifecycle.getClient())
        lifecycle.ensureInitialized()
        assertEquals(1, constructions)
        assertEquals(1, errors.size)
        assertEquals(if (stage == "construct") 0 else 1, client.closes.get())
        lifecycle.close()
    }

    private fun lifecycle(
        client: RecordingLifecycleClient,
        timeoutMillis: Long = 15_000,
        onFailure: (Exception) -> Unit = { throw AssertionError(it) }
    ) = ProjectClientLifecycle(
        { false }, { client }, RecordingLifecycleClient::initialize,
        RecordingLifecycleClient::notifyInitialized, timeoutMillis, onFailure
    )


    @Test
    fun `concurrent initialization callers wait for one fully initialized client`() {
        val client = RecordingLifecycleClient()
        val lifecycle = lifecycle(client)
        val first = CompletableFuture.runAsync { lifecycle.ensureInitialized() }
        val second = CompletableFuture.runAsync { lifecycle.ensureInitialized() }
        try {
            assertTrue(client.started.await(5, TimeUnit.SECONDS))
            assertNull(lifecycle.getClient())
            client.initialization.complete(null)
            first.get(5, TimeUnit.SECONDS)
            second.get(5, TimeUnit.SECONDS)
            assertSame(client, lifecycle.getClient())
            assertEquals(1, client.starts.get())
            assertEquals(1, client.notifications.get())
        } finally {
            client.initialization.complete(null)
            lifecycle.close()
            first.get(5, TimeUnit.SECONDS)
            second.get(5, TimeUnit.SECONDS)
        }
        assertEquals(1, client.closes.get())
    }
}
