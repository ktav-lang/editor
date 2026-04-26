// Ktav IntelliJ Platform plugin — build configuration.
//
// Uses the modern `org.jetbrains.intellij.platform` Gradle plugin (the
// successor to the legacy `org.jetbrains.intellij`). The plugin pulls the
// IntelliJ Platform IDE artifacts as ordinary dependencies, instead of
// downloading a sandbox SDK lazily at task-graph time.

plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.0.0"
    id("org.jetbrains.intellij.platform") version "2.2.0"
}

group = "lang.ktav"
version = providers.gradleProperty("pluginVersion").get()

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

        // Bundled TextMate plugin — provides the runtime engine that
        // tokenises files according to a `.tmLanguage.json` grammar. Our
        // KtavTextMateLoader registers the grammar shipped under
        // `resources/grammars/ktav/`.
        bundledPlugin("org.jetbrains.plugins.textmate")

        pluginVerifier()
        zipSigner()
        instrumentationTools()
    }

    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

intellijPlatform {
    pluginConfiguration {
        name = "Ktav"
        version = project.version.toString()
        ideaVersion {
            // IntelliJ 2024.3 is build 243; we keep an upper bound through
            // 2025.1 (251.*) — bump as we verify newer builds.
            sinceBuild = "243"
            untilBuild = "251.*"
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
            recommended()
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

tasks {
    wrapper {
        gradleVersion = "8.10"
    }
    test {
        useJUnitPlatform()
    }
}
