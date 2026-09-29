# TextMate Bundle Auto-Registration Problem in IntelliJ Plugins

> **Historical document.** Describes a rejected approach that tried to
> auto-register a bundled TextMate grammar at runtime. Since 0.2.0
> (2026-05-07) the plugin instead ships a native `KtavLexer` /
> `KtavSyntaxHighlighterFactory` (wired via `KtavParserDefinition`) for
> syntax highlighting — no TextMate bundle, no `KtavTextMateLoader`,
> no manual registration step. Kept for context on why that path was
> abandoned; statements below about a "current implementation" refer
> to the pre-0.2.0 attempt, not the shipped plugin.

## Summary

While developing the Ktav plugin for IntelliJ, we encountered a problem with automatically registering a bundled TextMate grammar. IntelliJ's TextMate API has significant limitations that make programmatic registration impossible or unreliable.

## What We Tried to Do

The goal was to enable syntax highlighting for `.ktav` files automatically when the plugin was installed, **without** requiring manual registration in the IDE settings.

## Solution Architecture

### Implementation of the Rejected Attempt (KtavTextMateLoader.kt, before 0.2.0)

```
1. appFrameCreated hook (IDE startup)
   ├─ Check for the bundle in the file system (development mode)
   ├─ If not found, extract it from the plugin JAR
   └─ Attempt to register it with TextMate

2. projectOpened hook (Project load)
   └─ Retry registration (for dynamic plugin loading)

3. Bundle extraction
   ├─ Find ktav-intellij-*.jar in lib/ (excluding searchableOptions)
   ├─ Extract grammars/ktav/ into a temporary directory
   └─ Create a .tmbundle directory with the correct structure

4. Registration attempt
   ├─ Attempt 1: Update textmate.xml (TextMateUserBundlesSettings)
   ├─ Attempt 2: Call TextMateService.readBundle()
   └─ Attempt 3: Call reloadEnabledBundles()
```

## What Worked ✓

1. **Bundle extraction from the JAR** - successfully produces a `.tmbundle`
2. **JAR filter** - correctly excludes searchableOptions.jar
3. **Lifecycle hooks** - both hooks fire as expected
4. **Reflection-based API calls** - successfully invoke TextMateService methods
5. **reloadEnabledBundles()** - runs without errors

## What Did Not Work ✗

### 1. TextMateService.readBundle() Returns null

```kotlin
val readBundleMethod = serviceCls.getMethod("readBundle", Path::class.java)
val bundle = readBundleMethod.invoke(service, bundlePath)
// → Result: null ❌
```

**Cause**: The bundle format does not meet the TextMate API's expectations. Possible reasons:
- Required files are missing (for example, `info.plist` or `menu.plist`)
- The directory structure is incorrect
- The API expects a different bundle format

### 2. Updating textmate.xml Works but Does Not Take Effect

Attempt 1: JSON as a simple key-value map
```json
{
  "ktav": "C:\\path\\to\\Ktav.tmbundle"
}
```
**Result**: `XmlSerializationException: Cannot deserialize TextMateUserBundleServiceState`

Attempt 2: JSON as an array of objects
```json
[{
  "name": "ktav",
  "enabled": true,
  "path": "C:\\path\\to\\Ktav.tmbundle"
}]
```
**Result**: Deserialization failed, with no errors logged

### 3. textmate.xml Is Unavailable During Registration

- The plugin initializes at **appFrameCreated** (very early)
- The IDE creates `textmate.xml` later
- The attempt to locate the file fails
- Even if the file is rewritten, the IDE might not reread it

### 4. The TextMate API Is Very Limited

Available `TextMateService` methods in WebStorm 2025.3:
```
- readBundle(Path)              → Bundle object or null
- reloadEnabledBundles()        → void (reloads already enabled bundles)
- getFileNameMatcherToScopeNameMapping()
- getLanguageDescriptorByExtension(String)
- getLanguageDescriptorByFileName(String)
- getShellVariableRegistry()
- getSnippetRegistry()
- getPreferenceRegistry()
```

**Missing**:
- `registerEnabledBundle()` - does not exist in 2025.3
- `enableBundle()` - does not exist
- `registerBundle()` - does not exist
- A public way to add a bundle to the enabled list

### 5. The TextMateBundleProvider Extension Point Does Not Exist (or Is Internal)

Attempt to use the extension point in plugin.xml:
```xml
<textmate.bundleProvider
  implementation="lang.ktav.KtavTextMateBundleProvider" />
```

**Result**: `Unresolved reference 'TextMateBundleProvider'` - the class is not exported through the public API

## Investigation of Other Plugins

### WDL IDE Plugin (Broad Institute)

Their implementation simply calls:
```java
TextMateService.getInstance().registerEnabledBundles(false);
```

**Problem**: `registerEnabledBundles()` does not register new bundles; it only reloads existing ones. It is unclear how they add a bundle to the enabled list.

### Other Plugins

Most plugins use TextMate bundles in one of these ways:
1. Through bundled_plugins.txt (for official JetBrains plugins)
2. By explicitly copying them into known directories
3. Without automatic registration, requiring users to add them manually

## TextMate Bundle Structure

The extracted bundle has the correct structure:
```
Ktav.tmbundle/
├── language-configuration.json
└── Syntaxes/
    └── ktav.tmLanguage.json
```

This matches the standard VS Code TextMate bundle format. However, the IDE might expect additional files or a different structure.

## TextMateUserBundlesSettings Structure

In WebStorm 2025.3, the configuration is stored at `~\AppData\Roaming\JetBrains\WebStorm2025.3\options\textmate.xml`:

```xml
<application>
  <component name="TextMateUserBundlesSettings">
    <![CDATA[{}]]>
  </component>
</application>
```

**How it works**:
- `TextMateUserBundlesSettings` is an AppState component
- Its contents are JSON wrapped in CDATA
- Manual bundle registration updates and rewrites the JSON
- The IDE reads the JSON at startup and calls `TextMateUserBundlesSettings.deserialize()`

**Problem**: The exact JSON schema expected by IDE version 2025.3 is unknown

## Why This Is Difficult

1. **Internal TextMate API** - the methods used are not part of the public IntelliJ SDK
2. **API changes between versions** - methods and signatures differ between 2024.x and 2025.x
3. **No documentation** - JetBrains does not document the TextMate plugin's internals
4. **Timing problems** - plugin initialization and IDE state require precise synchronization
5. **No extension point** - bundles cannot be declared in plugin.xml
6. **Version differences** - different IDE versions have different internal APIs

## Possible Solutions

### 1. Manual Registration with Good UX ✓ (Recommended at the Time)

**Advantages**:
- Works reliably
- Does not depend on internal APIs
- Is compatible with all IDE versions

**Implementation**:
- Extract the bundle automatically into a temporary directory
- Have the plugin offer the user a way to copy its path
- Alternatively, provide a "Register Ktav TextMate Bundle" menu action that opens Settings → TextMate Bundles
- Or provide detailed, step-by-step documentation

### 2. Write to textmate.xml Before the IDE Loads

**Difficulty**: The IDE initializes this setting very early, before the plugin loads.

**Possible solution**:
- Use `AppLifecycleListener.appStarting()` instead of `appFrameCreated()`
- Write to textmate.xml BEFORE the IDE reads it
- This requires precise timing and may be unstable

### 3. Use a Different Configuration Store

Instead of textmate.xml, write to:
- The project's `.idea/` directory (not compatible with global TextMate settings)
- A custom plugin configuration file (which the IDE will not read)

### 4. Bundled TextMate Bundles (Official Method)

**Requirements**:
- Register the bundle in bundled_plugins.txt
- This is possible only for official JetBrains plugins
- This method is unavailable to third-party developers

### 5. Wait for API Improvements in Future IDE Versions

**Status**: JetBrains might release a public extension point in 2026.x or later

## Recommendation

**Use solution 1 (manual registration with good UX).** This was the historical recommendation before the switch to the native lexer.

That meant:
1. ✓ Keep the then-current bundle extraction implementation, which worked well
2. ✓ Log the path to the extracted bundle
3. ✓ Create an IDE action or notification for the user
4. ✓ Write detailed documentation
5. ✓ Switch to automatic registration if JetBrains releases a public API in the future

## Code from the Rejected Attempt (Historical State, Before 0.2.0)

**Files** (absent from the current plugin, replaced by the native lexer):
- `src/main/kotlin/lang/ktav/KtavTextMateLoader.kt` - main logic
- `src/main/kotlin/lang/ktav/KtavProjectActivity.kt` - project lifecycle hook
- `src/main/resources/META-INF/plugin.xml` - plugin configuration

**Status when the approach was abandoned**:
- Bundle extraction: ✓ Worked
- Settings update: ⚠️ Ran but did not take effect
- Auto-registration: ✗ Could not be implemented reliably

Since 0.2.0, syntax highlighting has been provided by `KtavParserDefinition` and
`KtavSyntaxHighlighterFactory` (using the native `KtavLexer`); see
`intellij/src/main/resources/META-INF/plugin.xml`.

## References

- [IntelliJ TextMate Plugin Source](https://github.com/JetBrains/intellij-community/tree/master/plugins/textmate)
- [TextMate Bundle Format](https://macromates.com/manual/en/bundles)
- [WDL IDE Plugin](https://github.com/broadinstitute/wdl-ide)
- IntelliJ API: `org.jetbrains.plugins.textmate.TextMateService`
