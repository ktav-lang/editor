// Ktav IntelliJ Platform plugin — build configuration.
//
// Uses the modern `org.jetbrains.intellij.platform` Gradle plugin (the
// successor to the legacy `org.jetbrains.intellij`). The plugin pulls the
// IntelliJ Platform IDE artifacts as ordinary dependencies, instead of
// downloading a sandbox SDK lazily at task-graph time.

import java.text.SimpleDateFormat
import java.util.Date

plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.0.0"
    id("org.jetbrains.intellij.platform") version "2.2.0"
}

group = "lang.ktav"
// Version + build timestamp so users can see in IDE whether the installed
// build is fresh. Format: "0.1.5+20260507-1638"
version = providers.gradleProperty("pluginVersion").get() +
    "+" + SimpleDateFormat("yyyyMMdd-HHmm").format(Date())

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        // Target IDE: IntelliJ IDEA Community 2024.3 (build 243.x). All
        // other JetBrains IDEs that share the platform (RustRover, GoLand,
        // WebStorm, PyCharm, ...) at this build number can load the plugin
        // too — see `pluginVerification` below.
        intellijIdeaCommunity("2024.3")

        pluginVerifier()
        zipSigner()
        instrumentationTools()

        // Test framework for `BasePlatformTestCase` — required by the
        // commenter-toggle / file-type assertions in `KtavPlatformTest`.
        testFramework(org.jetbrains.intellij.platform.gradle.TestFrameworkType.Platform)
    }

    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
    // BasePlatformTestCase derives from the JUnit 3 / 4 lineage; pull JUnit 4
    // for the heavyweight platform test only.
    testImplementation("junit:junit:4.13.2")
    // Run JUnit 4 BasePlatformTestCase tests under the JUnit Platform.
    testRuntimeOnly("org.junit.vintage:junit-vintage-engine:5.10.0")
}

intellijPlatform {
    pluginConfiguration {
        name = "Ktav"
        version = project.version.toString()
        ideaVersion {
            // Support IntelliJ/JetBrains IDEs from 2021.1 (build 211) forward.
            // This includes: IntelliJ IDEA, WebStorm, PyCharm, PhpStorm, RubyMine,
            // CLion, GoLand, RustRover, Rider, and all other platform-based IDEs
            // from 2021.1 through 2025.x (251+).
            //
            // Note on dynamic plugin loading (unload without IDE restart):
            // - Supported in 2023.2+ (build 232)
            // - For 2021.1-2023.1, plugins require IDE restart to unload
            // - This is an IDE limitation, not a plugin limitation
            //
            // No upper bound — rely on `pluginVerifier` (run in CI) to catch
            // breakage on newer builds instead of pinning to a version that
            // blocks newer IDEs.
            sinceBuild = "211"
            // Explicitly set empty until-build to prevent gradle-intellij-platform
            // from auto-adding "211.*" constraint
            untilBuild = ""
        }
        description = """
            <p>Editor support for the
            <a href="https://github.com/ktav-lang/spec">Ktav</a>
            plain configuration format.</p>

            <p>Features:</p>
            <ul>
              <li>Syntax highlighting for <code>.ktav</code> files via TextMate grammar</li>
              <li>Comment toggle (<code>#</code>)</li>
              <li>Bracket matching for <code>{}</code> <code>[]</code> <code>()</code></li>
              <li>Live diagnostics via
                <a href="https://github.com/ktav-lang/editor/tree/main/lsp">ktav-lsp</a>
                (when installed in PATH; LSP integration arrives in a later release)</li>
            </ul>
        """.trimIndent()
        changeNotes = providers.fileContents(layout.projectDirectory.file("CHANGELOG.md"))
            .asText
            .map { it.lines().take(20).joinToString("\n") }
    }
    pluginVerification {
        ides {
            // Pin to known-published IDE versions instead of `recommended()`.
            // The latter follows JetBrains metadata which sometimes lists
            // the upcoming release before its artefact is uploaded — that
            // breaks the verifier with `Could not find idea:ideaIC:X.Y`.
            // The pinned set covers our since-build floor (2024.3 / 243)
            // through the latest published stable.
            ide("IC-2024.3")
            ide("IC-2025.1")
            ide("IC-2025.2")
        }
    }
    publishing {
        // Marketplace upload uses a personal access token (PAT) generated
        // at https://plugins.jetbrains.com/author/me/tokens. CI-only.
        token = providers.environmentVariable("INTELLIJ_PUBLISH_TOKEN")
    }
}

kotlin {
    jvmToolchain(17)
}

// ----------------------------------------------------------------------
// Grammar sync
// ----------------------------------------------------------------------
//
// The TextMate grammar lives once at `editor/grammars/`. We copy it into
// `src/main/resources/grammars/ktav/` so it ends up inside the packaged
// plugin jar. The same destination structure (a `Syntaxes/` subfolder
// containing the `.tmLanguage.json`) is what the IntelliJ TextMate engine
// expects when registering a bundle.
val syncGrammars = tasks.register<Copy>("syncGrammars") {
    description = "Mirror the shared TextMate grammar into resources/grammars/ktav/"
    group = "build"

    val grammarsDir = layout.projectDirectory.dir("../grammars")
    from(grammarsDir.file("ktav.tmLanguage.json")) {
        into("Syntaxes")
    }
    from(grammarsDir.file("language-configuration.json"))
    into(layout.projectDirectory.dir("src/main/resources/grammars/ktav"))
}

tasks.named("processResources") {
    dependsOn(syncGrammars)
}

tasks.named("compileKotlin") {
    dependsOn(syncGrammars)
}

// Include bin/ directory (with pre-built LSP binaries) in the plugin distribution.
// Copy to sandbox for development (IDE testing).
tasks.named("prepareSandbox") {
    doLast {
        copy {
            from(layout.projectDirectory.dir("bin"))
            into(layout.buildDirectory.dir("idea-sandbox/plugins/ktav-intellij/lib/bin"))
        }
    }
}

// Post-process the built plugin ZIP to include LSP binaries using Gradle's Zip task.
// gradle-intellij-platform's buildPlugin task doesn't include custom files, so we
// extract, add binaries, and re-zip the distribution.

val tempDistDir = layout.buildDirectory.dir("plugin-dist-with-bins")

tasks.register("_extractAndAddBinaries") {
    dependsOn("buildPlugin")

    inputs.file(layout.buildDirectory.file("distributions/ktav-intellij-${project.version}.zip"))
    inputs.dir(layout.projectDirectory.dir("bin"))
    outputs.dir(tempDistDir)

    doLast {
        val originalZip = layout.buildDirectory.file("distributions/ktav-intellij-${project.version}.zip").get().asFile
        val tempDir = tempDistDir.get().asFile
        val binDir = layout.projectDirectory.dir("bin").asFile

        println(">>> Extracting and adding binaries...")

        // Cleanup and prepare temp directory
        if (tempDir.exists()) {
            tempDir.deleteRecursively()
        }
        tempDir.mkdirs()

        // Extract original ZIP
        copy {
            from(zipTree(originalZip))
            into(tempDir)
        }
        println(">>> Extracted original ZIP")

        // Copy binaries into extracted structure
        copy {
            from(binDir)
            into(File(tempDir, "ktav-intellij/lib/bin"))
        }
        println(">>> Copied binaries to ktav-intellij/lib/bin")
    }
}

tasks.register<Zip>("_repackageWithBinaries") {
    dependsOn("_extractAndAddBinaries")

    from(tempDistDir.get().dir("ktav-intellij")) {
        into("ktav-intellij")
    }

    archiveFileName.set("ktav-intellij-${project.version}.zip")
    destinationDirectory.set(layout.buildDirectory.dir("distributions"))

    doFirst {
        // Remove old distribution so our new one can take its place
        layout.buildDirectory.file("distributions/ktav-intellij-${project.version}.zip").get().asFile.delete()
    }

    doLast {
        println(">>> Repackaged plugin with binaries")
        val zipFile = archiveFile.get().asFile
        println(">>> Archive size: ${zipFile.length()} bytes")

        // Cleanup temporary directory
        tempDistDir.get().asFile.deleteRecursively()
        println(">>> Cleaned up temporary directory")
    }
}

// Make buildPlugin task run the repackaging automatically
tasks.named("buildPlugin") {
    finalizedBy("_repackageWithBinaries")
}

tasks {
    wrapper {
        gradleVersion = "8.10"
    }
    test {
        useJUnitPlatform()
    }

}
