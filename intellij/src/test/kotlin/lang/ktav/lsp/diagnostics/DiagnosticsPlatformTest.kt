package lang.ktav.lsp.diagnostics

import com.google.gson.JsonParser
import com.intellij.codeInsight.daemon.impl.AnnotationHolderImpl
import com.intellij.lang.annotation.AnnotationSession
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.psi.PsiManager
import com.intellij.psi.PsiDocumentManager
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.CommandProcessor
import com.intellij.openapi.editor.Document
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.editor.EditorFactory
import com.intellij.openapi.module.EmptyModuleType
import com.intellij.openapi.module.ModuleManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectLocator
import com.intellij.openapi.project.ex.ProjectManagerEx
import com.intellij.openapi.roots.ModuleRootModificationUtil
import com.intellij.openapi.roots.ProjectFileIndex
import com.intellij.testFramework.IndexingTestUtil
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.RunAll
import com.intellij.testFramework.createTestOpenProjectOptions
import com.intellij.testFramework.fixtures.impl.TempDirTestFixtureImpl
import java.nio.file.Paths
import lang.ktav.lsp.LocalPlatformTestCase
import lang.ktav.lsp.UriUtil
import lang.ktav.lsp.lifecycle.ChangeTracker
import lang.ktav.lsp.lifecycle.DocumentSyncConnection
import lang.ktav.lsp.lifecycle.getLspService

/**
 * Runs real project services, queued UI callbacks, Editors and MarkupModels headlessly.
 * Smoke: gradlew.bat test --tests lang.ktav.lsp.diagnostics.DiagnosticsPlatformTest --rerun-tasks
 */
class DiagnosticsPlatformTest : LocalPlatformTestCase() {
    private val extraEditors = mutableListOf<Editor>()
    private val extraProjects = mutableListOf<Project>()
    private val physicalFiles = TempDirTestFixtureImpl()

    override fun setUp() {
        super.setUp()
        physicalFiles.setUp()
    }

    private class Owner(val project: Project, val uri: String, val document: Document) {
        val tracker = ChangeTracker.getInstance(project)
        val service = project.getLspService()
        val client = Any()
        var active = true
        var version = 0
        private val connection = DocumentSyncConnection(client,
            { _, v, _ -> version = v }, { _, v, _ -> version = v }, {}, { active })
        fun open() = tracker.attach(uri, document, connection)
        fun close() = tracker.detach(uri)
        fun publish(message: String?, v: Int? = version, identity: Any = client): Boolean {
            val diagnostic = if (message == null) "[]" else """[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"message":"$message","severity":1}]"""
            val versionField = if (v == null) "" else ",\"version\":$v"
            val params = JsonParser.parseString("""{"uri":"$uri"$versionField,"diagnostics":$diagnostic}""")
            return ApplicationManager.getApplication().runReadAction<Boolean> {
                service.publishDiagnostics(identity, params)
            }
        }
        fun diagnostics() = DiagnosticsHolder.getInstance(project).getDiagnostics(uri).map { it.message }
    }

    private fun document(): Pair<String, Document> {
        // A separate physical root is not owned by the parent light fixture.
        val file = physicalFiles.createFile("shared/src/shared.txt", "v: 1")
        val document = FileDocumentManager.getInstance().getDocument(file)!!
        return UriUtil.fromVirtualFile(file) to document
    }

    private fun otherProject(document: Document): Project {
        val file = FileDocumentManager.getInstance().getFile(document)!!
        val name = "diagnostics-${extraProjects.size}"
        val path = ApplicationManager.getApplication().runWriteAction<String> {
            physicalFiles.findOrCreateDir("owners/$name").path
        }
        // Use the SDK's headless opening options, not an unregistered project.
        // ProjectLocator must see both genuine owners of the same physical file.
        val owner = ProjectManagerEx.getInstanceEx()
            .openProject(Paths.get(path), createTestOpenProjectOptions())!!
        extraProjects.add(owner)
        ApplicationManager.getApplication().runWriteAction {
            val module = ModuleManager.getInstance(owner)
                .newModule(Paths.get(path, "$name.iml"), EmptyModuleType.EMPTY_MODULE)
            ModuleRootModificationUtil.addContentRoot(module, file.parent)
        }
        IndexingTestUtil.waitUntilIndexesAreReady(owner)
        assertTrue(owner.isOpen)
        assertTrue(owner.isInitialized)
        assertTrue(ProjectFileIndex.getInstance(owner).isInContent(file))
        assertContainsElements(ProjectLocator.getInstance().getProjectsForFile(file), owner)
        val psi = PsiManager.getInstance(owner).findFile(file)!!
        assertSame(owner, psi.project)
        assertSame(document, PsiDocumentManager.getInstance(owner).getDocument(psi))
        return owner
    }

    private fun editor(document: Document, owner: Project): Editor =
        EditorFactory.getInstance().createEditor(document, owner).also {
            extraEditors.add(it)
            assertSame(owner, it.project)
            assertSame(document, it.document)
        }

    private fun disposeProject(owner: Project) {
        PlatformTestUtil.forceCloseProjectWithoutSaving(owner)
        assertTrue(owner.isDisposed)
        extraProjects.remove(owner)
    }

    private fun flush() = PlatformTestUtil.dispatchAllInvocationEventsInIdeEventQueue()

    private fun marks(editor: Editor, message: String) =
        editor.markupModel.allHighlighters.filter { it.errorStripeTooltip == message }

    private fun edit(document: Document, text: String) {
        CommandProcessor.getInstance().runUndoTransparentAction {
            ApplicationManager.getApplication().runWriteAction { document.setText(text) }
        }
    }

    fun testHeadlessDiagnosticSurfaceKeepsProjectsAndEveryEditorIndependent() {
        val (uri, document) = document()
        val a = Owner(otherProject(document), uri, document)
        val b = Owner(otherProject(document), uri, document)
        val a1 = editor(document, a.project)
        val a2 = editor(document, a.project)
        val b1 = editor(document, b.project)
        a.open()
        b.open()
        assertTrue(a.publish("owned-A"))
        assertTrue(b.publish("owned-B"))
        flush()
        val ha1 = marks(a1, "owned-A").single()
        val ha2 = marks(a2, "owned-A").single()
        val hb = marks(b1, "owned-B").single()
        assertEquals(0, ha1.startOffset)
        assertEquals(1, ha1.endOffset)
        assertEmpty(marks(a1, "owned-B"))
        assertEmpty(marks(b1, "owned-A"))
        val file = FileDocumentManager.getInstance().getFile(document)!!
        val annotator = KtavDiagnosticsAnnotator()
        val psiA = PsiManager.getInstance(a.project).findFile(file)!!
        val psiB = PsiManager.getInstance(b.project).findFile(file)!!
        assertEquals(listOf("owned-A"), annotator.doAnnotate(psiA)!!.diagnostics.map { it.message })
        assertEquals(listOf("owned-B"), annotator.doAnnotate(psiB)!!.diagnostics.map { it.message })
        val wrongOwnerAnnotations = AnnotationHolderImpl(AnnotationSession(psiB), false)
        annotator.apply(psiB, annotator.doAnnotate(psiA)!!, wrongOwnerAnnotations)
        assertEmpty(wrongOwnerAnnotations)
        val superseded = annotator.doAnnotate(psiA)!!
        assertTrue(a.publish("updated-A"))
        val supersededAnnotations = AnnotationHolderImpl(AnnotationSession(psiA), false)
        annotator.apply(psiA, superseded, supersededAnnotations)
        assertEmpty(supersededAnnotations)
        flush()
        assertFalse(ha1.isValid)
        assertFalse(ha2.isValid)
        assertTrue(hb.isValid)
        val updated1 = marks(a1, "updated-A").single()
        val updated2 = marks(a2, "updated-A").single()

        a.close()
        assertFalse(a.publish(null, null)) // Delayed unversioned didClose notification.
        flush()
        assertFalse(updated1.isValid)
        assertFalse(updated2.isValid)
        assertTrue(hb.isValid)
        assertEquals(listOf("owned-B"), b.diagnostics())
        assertEmpty(a.diagnostics())
        assertTrue(b.publish(null))
        flush()
        assertFalse(hb.isValid)
        println("DIAGNOSTICS_SURFACE: two projects/shared URI; both A models cleared; B preserved then cleared")
    }

    fun testDelayedVersionSessionClientAndDisposedPublicationsCannotRender() {
        val (uri, document) = document()
        val owner = Owner(otherProject(document), uri, document)
        val target = editor(document, owner.project)
        owner.open()
        val initialVersion = owner.version
        assertTrue(owner.publish("before-edit"))
        val file = FileDocumentManager.getInstance().getFile(document)!!
        val psi = PsiManager.getInstance(owner.project).findFile(file)!!
        val annotator = KtavDiagnosticsAnnotator()
        val collected = annotator.doAnnotate(psi)!!
        edit(document, "v: 2") // Also invalidates an already queued renderer callback.
        assertFalse(owner.publish("stale-version", initialVersion))
        val annotations = AnnotationHolderImpl(AnnotationSession(psi), false)
        annotator.apply(psi, collected, annotations)
        assertEmpty(annotations)
        flush()
        assertEmpty(marks(target, "before-edit"))
        assertEmpty(marks(target, "stale-version"))
        assertTrue(owner.publish("current"))
        flush()
        val current = marks(target, "current").single()
        val closedVersion = owner.version
        owner.close()
        assertFalse(owner.publish("closed", closedVersion))
        owner.open()
        assertFalse(owner.publish("reopened-stale", closedVersion))
        assertFalse(owner.publish(null, null))
        assertFalse(owner.publish("wrong-client", identity = Any()))
        assertTrue(owner.publish("reopened-current"))
        flush()
        assertFalse(current.isValid)
        val reopened = marks(target, "reopened-current").single()
        owner.active = false
        assertFalse(owner.publish("inactive-client"))
        assertEmpty(owner.diagnostics())
        val replacement = Owner(owner.project, uri, document)
        replacement.open()
        assertFalse(owner.publish("old-client", replacement.version))
        assertTrue(replacement.publish("replacement-client"))
        flush()
        assertFalse(reopened.isValid)
        val replacementHighlight = marks(target, "replacement-client").single()
        owner.service.close()
        assertFalse(owner.publish("disposed-service"))
        flush()
        assertFalse(reopened.isValid)
        assertFalse(replacementHighlight.isValid)
        assertEmpty(target.markupModel.allHighlighters.filter { it.errorStripeTooltip == "disposed-service" })
    }

    fun testReleasingOneEditorAndDisposingOwnerClearsOnlyOwnedHighlighters() {
        val (uri, document) = document()
        val a = Owner(otherProject(document), uri, document)
        val b = Owner(otherProject(document), uri, document)
        val a1 = editor(document, a.project)
        val a2 = editor(document, a.project)
        val b1 = editor(document, b.project)
        a.open()
        b.open()
        assertTrue(a.publish("A"))
        assertTrue(b.publish("B"))
        flush()
        val h1 = marks(a1, "A").single()
        val h2 = marks(a2, "A").single()
        val hb = marks(b1, "B").single()
        val foreign = a2.markupModel.addRangeHighlighter(2, 3, 1, null,
            com.intellij.openapi.editor.markup.HighlighterTargetArea.EXACT_RANGE)
        extraEditors.remove(a1)
        EditorFactory.getInstance().releaseEditor(a1)
        assertFalse(h1.isValid)
        assertTrue(h2.isValid)
        a.service.close()
        flush()
        assertFalse(h2.isValid)
        assertTrue(foreign.isValid)
        assertTrue(hb.isValid)

        // Dispose with a render callback still queued; it must not republish.
        assertTrue(b.publish("queued-before-disposal"))
        extraEditors.remove(b1)
        EditorFactory.getInstance().releaseEditor(b1)
        disposeProject(b.project)
        assertFalse(b.publish("disposed-project"))
        flush()
        assertFalse(hb.isValid)
        assertTrue(foreign.isValid)
    }

    override fun tearDown() {
        RunAll.runAll(
            {
                RunAll.runAll(extraEditors) {
                    if (!it.isDisposed) EditorFactory.getInstance().releaseEditor(it)
                }
            },
            { extraEditors.clear() },
            {
                RunAll.runAll(extraProjects.toList().asReversed()) {
                    if (!it.isDisposed) disposeProject(it)
                }
            },
            { extraProjects.clear() },
            { flush() },
            { physicalFiles.tearDown() },
            { super.tearDown() },
        )
    }
}
