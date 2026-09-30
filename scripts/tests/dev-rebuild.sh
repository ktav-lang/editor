#!/usr/bin/env bash
# Real filesystem/archive scenarios; build tools produce disposable fixture artifacts.
# Never touches a real IDE, its cache, a checkout build output, or running processes.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_ROOT="$(cd -- "${TMPDIR:-/tmp}" && pwd -P)"
SUITE="$(mktemp -d "$TMP_ROOT/ktav-rebuild-smoke.XXXXXX")"
cleanup() {
    [[ -d "$SUITE" && ! -L "$SUITE" && "$SUITE" == "$TMP_ROOT"/ktav-rebuild-smoke.* ]] || return 1
    rm -rf -- "$SUITE"
}
trap cleanup EXIT
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
mkdir -p "$SUITE/checkout/intellij" "$SUITE/checkout/lsp" "$SUITE/tools" "$SUITE/caller" \
    "$SUITE/plugins/ktav-intellij/lib/bin/linux-x64" "$SUITE/plugins/other-plugin" "$SUITE/cache/caches"
cp -- "$ROOT/intellij/dev-rebuild.sh" "$SUITE/checkout/intellij/"
printf old > "$SUITE/plugins/ktav-intellij/lib/bin/linux-x64/ktav-lsp"
printf obsolete > "$SUITE/plugins/ktav-intellij/lib/old.jar"
printf unrelated > "$SUITE/plugins/other-plugin/plugin.jar"
printf unrelated-zip > "$SUITE/plugins/unrelated.zip"
printf cache > "$SUITE/cache/caches/sentinel"
cp -R -- "$SUITE/plugins/other-plugin" "$SUITE/before-other"
cp -R -- "$SUITE/cache" "$SUITE/before-cache"
cp -- "$SUITE/plugins/unrelated.zip" "$SUITE/before.zip"
cat > "$SUITE/tools/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${FAIL_BUILD:-0}" == 1 ]]; then exit 42; fi
mkdir -p "$FIXTURE/checkout/lsp/target/x86_64-unknown-linux-gnu/release"
printf fresh > "$FIXTURE/checkout/lsp/target/x86_64-unknown-linux-gnu/release/ktav-lsp"
chmod +x "$FIXTURE/checkout/lsp/target/x86_64-unknown-linux-gnu/release/ktav-lsp"
CARGO
cat > "$SUITE/checkout/intellij/gradlew" <<'GRADLE'
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$FIXTURE/archive/ktav-intellij/lib/bin/linux-x64" "$FIXTURE/checkout/intellij/build/distributions"
cp "$FIXTURE/checkout/intellij/bin/linux-x64/ktav-lsp" "$FIXTURE/archive/ktav-intellij/lib/bin/linux-x64/"
printf new-jar > "$FIXTURE/archive/ktav-intellij/lib/new.jar"
python3 -m zipfile -c "$FIXTURE/checkout/intellij/build/distributions/ktav-intellij-0.8.0.zip" "$FIXTURE/archive/ktav-intellij"
GRADLE
chmod +x "$SUITE/tools/cargo" "$SUITE/checkout/intellij/gradlew"
export FIXTURE="$SUITE"
export PATH="$SUITE/tools:$PATH"
unset KTAV_PLUGINS_DIR KTAV_PLATFORM KTAV_PLUGIN_ZIP KTAV_IDE_EXECUTABLE KTAV_IDE_CACHE_DIR
helper="$SUITE/checkout/intellij/dev-rebuild.sh"

# Run outside the copied checkout: missing configuration must fail before building.
if (cd "$SUITE/caller" && bash "$helper" --no-restart); then fail 'missing configuration accepted'; fi
[[ ! -e "$SUITE/checkout/lsp/target" ]] || fail 'build occurred before configuration validation'

# Binary-only update must use this script's sibling lsp/ and preserve plugin JARs.
(cd "$SUITE/caller" && bash "$helper" --plugins-dir "$SUITE/plugins" --platform linux-x64 --cache-dir "$SUITE/cache" --no-restart)
[[ "$(< "$SUITE/plugins/ktav-intellij/lib/bin/linux-x64/ktav-lsp")" == fresh ]] || fail 'selected plugin binary not updated'
[[ "$(< "$SUITE/plugins/ktav-intellij/lib/old.jar")" == obsolete ]] || fail 'binary update replaced plugin JARs'

# Full replacement removes obsolete Ktav files, not unrelated plugins/ZIPs/cache.
(cd "$SUITE/caller" && KTAV_PLUGINS_DIR="$SUITE/plugins" KTAV_PLATFORM=linux-x64 KTAV_IDE_CACHE_DIR="$SUITE/cache" bash "$helper" --plugin --no-restart)
[[ -f "$SUITE/plugins/ktav-intellij/lib/new.jar" && ! -e "$SUITE/plugins/ktav-intellij/lib/old.jar" ]] || fail 'Ktav installation was not replaced'
[[ "$(< "$SUITE/plugins/ktav-intellij/lib/bin/linux-x64/ktav-lsp")" == fresh ]] || fail 'replacement lost LSP binary'
[[ -x "$SUITE/plugins/ktav-intellij/lib/bin/linux-x64/ktav-lsp" ]] || fail 'replacement lost executable permission'
diff -r "$SUITE/before-other" "$SUITE/plugins/other-plugin" || fail 'unrelated plugin changed'
cmp "$SUITE/before.zip" "$SUITE/plugins/unrelated.zip" || fail 'unrelated ZIP changed'
diff -r "$SUITE/before-cache" "$SUITE/cache" || fail 'IDE cache changed'
cp -R -- "$SUITE/plugins/ktav-intellij" "$SUITE/before-ktav"

# A failed compiler must leave the installed plugin intact.
status=0
(cd "$SUITE/caller" && FAIL_BUILD=1 bash "$helper" --plugins-dir "$SUITE/plugins" --platform linux-x64 --plugin --no-restart) || status=$?
[[ "$status" == 42 ]] || fail 'compiler failure was hidden'
diff -r "$SUITE/before-ktav" "$SUITE/plugins/ktav-intellij" || fail 'installation changed after build failure'

# A supplied archive cannot install a sibling plugin or write outside Ktav.
mkdir -p "$SUITE/bad/other-plugin"
printf unwanted > "$SUITE/bad/other-plugin/injected"
python3 -m zipfile -c "$SUITE/bad.zip" "$SUITE/bad/other-plugin"
if (cd "$SUITE/caller" && bash "$helper" --plugins-dir "$SUITE/plugins" --platform linux-x64 --plugin --plugin-zip "$SUITE/bad.zip" --no-restart); then fail 'foreign archive accepted'; fi
diff -r "$SUITE/before-ktav" "$SUITE/plugins/ktav-intellij" || fail 'installation changed after foreign archive'
diff -r "$SUITE/before-other" "$SUITE/plugins/other-plugin" || fail 'foreign archive affected another plugin'
cmp "$SUITE/before.zip" "$SUITE/plugins/unrelated.zip" || fail 'ZIP deleted during replacement'
diff -r "$SUITE/before-cache" "$SUITE/cache" || fail 'cache changed during replacement'
printf 'PASS: rebuild paths, isolated installation, failure preservation, archive boundaries\n'
