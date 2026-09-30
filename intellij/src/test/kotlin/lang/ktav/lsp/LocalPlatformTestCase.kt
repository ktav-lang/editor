package lang.ktav.lsp

import com.intellij.openapi.util.Disposer
import com.intellij.openapi.util.registry.Registry
import com.intellij.testFramework.LightProjectDescriptor
import com.intellij.testFramework.TestApplicationManager
import com.intellij.testFramework.fixtures.BasePlatformTestCase

/** Platform fixtures for local documents, not WSL filesystem integration. */
abstract class LocalPlatformTestCase : BasePlatformTestCase() {
    private val localFilesystem = Disposer.newDisposable("Ktav local filesystem fixture")

    // LightPlatformTestCase caches projects by descriptor equality. A new identity for every
    // fixture setup forces a fresh Project and its services, including terminally disposable LSP state.
    override fun getProjectDescriptor(): LightProjectDescriptor = LightProjectDescriptor()

    override fun setUp() {
        TestApplicationManager.getInstance()
        // IJPL-178929: the WSL test service class is not shipped in the external SDK.
        // These fixtures exercise local documents only; disable the unrelated WSL route before project startup.
        Registry.get("wsl.use.remote.agent.for.nio.filesystem").setValue(false, localFilesystem)
        try {
            super.setUp()
        } catch (failure: Throwable) {
            Disposer.dispose(localFilesystem)
            throw failure
        }
    }

    override fun tearDown() {
        try {
            super.tearDown()
        } finally {
            Disposer.dispose(localFilesystem)
        }
    }
}
