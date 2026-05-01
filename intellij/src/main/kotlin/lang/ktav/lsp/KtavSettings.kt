package lang.ktav.lsp

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.util.xmlb.XmlSerializerUtil

/**
 * Application-level persisted settings for Ktav LSP integration.
 *
 * Currently a single field: the explicit path to the `ktav-lsp` server
 * binary. Empty means "fall through to the discovery chain" (bundled
 * binary -> PATH).
 */
@Service(Service.Level.APP)
@State(
    name = "KtavSettings",
    storages = [Storage("ktav.xml")],
)
class KtavSettings : PersistentStateComponent<KtavSettings.State> {

    data class State(
        var serverPath: String = "",
    )

    private var internalState: State = State()

    override fun getState(): State = internalState

    override fun loadState(state: State) {
        XmlSerializerUtil.copyBean(state, internalState)
    }

    companion object {
        @JvmStatic
        fun getInstance(): KtavSettings =
            ApplicationManager.getApplication().getService(KtavSettings::class.java)
    }
}
