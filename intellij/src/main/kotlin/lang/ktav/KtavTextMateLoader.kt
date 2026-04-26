package lang.ktav

import com.intellij.ide.AppLifecycleListener
import com.intellij.openapi.diagnostic.Logger

/**
 * Registers the bundled Ktav TextMate grammar with IntelliJ's TextMate
 * plugin at IDE startup.
 *
 * Background
 * ----------
 * The TextMate plugin in IntelliJ exposes two registration paths:
 *
 *  1. **User-discovered bundles** — folders the user adds via
 *     Settings → Editor → TextMate Bundles. These get persisted in
 *     `TextMateUserBundlesSettings` and loaded on startup.
 *
 *  2. **Built-in bundles** — the engine also auto-discovers any bundle
 *     under its `bundles/` resource root, but the contract for a third-
 *     party plugin to inject one is unstable across IDE versions
 *     (`TextMateService` was internalised in 2024.x; the public
 *     `TextMateBundleProvider` SPI shipped in some builds and was renamed
 *     in others). Programmatic registration via reflection works but is
 *     brittle.
 *
 * Approach we take
 * ----------------
 * We ship the grammar inside the plugin jar at
 * `resources/grammars/ktav/Syntaxes/ktav.tmLanguage.json` (a minimal
 * TextMate-bundle layout — the `Syntaxes/` subdir is the canonical name
 * the engine looks for). For a robust first release we leave runtime
 * registration as a TODO and verify the file-type + commenter wiring
 * works without highlighting first; the next iteration will either:
 *
 *   a) call `TextMateService.getInstance().registerEnabledBundle(path)`
 *      after extracting the bundle to a temp dir, or
 *   b) implement the `TextMateBundleProvider` extension point
 *      (preferred, but only works on builds that ship the SPI).
 *
 * Until then the user can still add the bundle manually via the TextMate
 * settings UI by pointing it at the unpacked plugin folder under
 * `<plugins>/Ktav/lib/grammars/ktav/`.
 */
class KtavTextMateLoader : AppLifecycleListener {

    private val log = Logger.getInstance(KtavTextMateLoader::class.java)

    override fun appFrameCreated(commandLineArgs: List<String>) {
        // TODO(textmate): register `resources/grammars/ktav/` with the
        //   bundled TextMate plugin so `.ktav` files highlight without
        //   the user having to add the bundle by hand. See class kdoc
        //   for the two candidate APIs and their portability trade-offs.
        log.info("Ktav plugin loaded; TextMate auto-registration pending (see KtavTextMateLoader).")
    }
}
