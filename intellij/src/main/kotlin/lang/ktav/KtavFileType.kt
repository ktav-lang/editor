package lang.ktav

import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

/**
 * File type for `.ktav` files.
 *
 * Registered via plugin.xml with `fieldName="INSTANCE"` so the IntelliJ
 * Platform can resolve the singleton through Kotlin's `INSTANCE` field
 * (the JVM bytecode shape of a Kotlin `object`).
 *
 * TODO: ship a real 16x16 SVG/PNG icon and return it from [getIcon].
 *       For now we return null, which makes the IDE fall back to the
 *       generic-text-file icon.
 */
object KtavFileType : LanguageFileType(KtavLanguage) {
    init {
        com.intellij.openapi.diagnostic.Logger.getInstance(KtavFileType::class.java)
            .info("[Ktav FileType] KtavFileType object created (extension='ktav')")
        println(">>> [Ktav FileType] KtavFileType object created")
    }

    override fun getName(): String = "Ktav"
    override fun getDescription(): String = "Ktav configuration file"
    override fun getDefaultExtension(): String = "ktav"
    override fun getIcon(): Icon? = null
}
