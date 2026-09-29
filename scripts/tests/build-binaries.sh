#!/bin/bash
# Regression fixtures only; no Rust compilation or containers.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
TMP_DIR="$(cd "${TMPDIR:-/tmp}" && pwd -P)"
SUITE="$(mktemp -d "${TMP_DIR%/}/build-binaries-tests.XXXXXX")"
SUITE="$(cd "$SUITE" && pwd -P)"
cleanup() {
    local status=$?
    trap - EXIT
    [[ ! -L "$SUITE" && -d "$SUITE" &&
       "$SUITE" == "${TMP_DIR%/}"/build-binaries-tests.* &&
       "$(cd "$SUITE/.." && pwd -P)" == "$TMP_DIR" ]] || return 1
    rm -rf -- "$SUITE" || return 1
    return "$status"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

TARGETS=(
    x86_64-pc-windows-msvc:win32-x64:ktav-lsp.exe
    aarch64-pc-windows-msvc:win32-arm64:ktav-lsp.exe
    x86_64-unknown-linux-gnu:linux-x64:ktav-lsp
    aarch64-unknown-linux-gnu:linux-arm64:ktav-lsp
    x86_64-apple-darwin:darwin-x64:ktav-lsp
    aarch64-apple-darwin:darwin-arm64:ktav-lsp
)
REAL_CP="$(command -v cp)"
REAL_MV="$(command -v mv)"
export REAL_CP REAL_MV

fail() {
    echo "FAIL: $*" >&2
    if [[ -f "${fixture:-}/build.log" ]]; then
        cat "$fixture/build.log" >&2
    fi
    exit 1
}

setup() {
    fixture="$SUITE/$1 fixture"
    scratch="$SUITE/$1 scratch"
    mkdir -p "$fixture/scripts" "$fixture/lsp" "$fixture/tools" "$scratch"
    cp "$ROOT_DIR/scripts/build-binaries.sh" "$fixture/scripts/"
    for spec in "${TARGETS[@]}"; do
        IFS=: read -r target platform binary <<< "$spec"
        mkdir -p "$fixture/lsp/target/$target/release"
        printf 'stale:%s\n' "$target" > "$fixture/lsp/target/$target/release/$binary"
        for plugin in vscode intellij; do
            mkdir -p "$fixture/$plugin/bin/$platform"
            printf 'old:%s:%s\n' "$plugin" "$target" > "$fixture/$plugin/bin/$platform/$binary"
        done
    done
    cp -R "$fixture/vscode/bin" "$fixture/before-vscode"
    cp -R "$fixture/intellij/bin" "$fixture/before-intellij"
    cat > "$fixture/tools/cargo" <<'STUB'
#!/bin/bash
set -euo pipefail
tool="${0##*/}"
printf '%s %s\n' "$tool" "$*" >> "$FIXTURE/calls"
[[ $# == 7 && $1 == build && $2 == --locked && $3 == --release &&
   $4 == --target && $6 == --target-dir ]] || exit 90
target="$5"
target_dir="$7"
[[ "$target_dir" == "$SCRATCH"/build-binaries.*/target ]] || exit 91
if [[ "$target" == *windows* ]]; then
    [[ "$tool" == cargo ]] || exit 92
    binary=ktav-lsp.exe
else
    [[ "$tool" == cross ]] || exit 93
    binary=ktav-lsp
fi
if [[ "$target" == "${FAIL_TARGET:-}" ]]; then
    # Even output plus a misleading success line cannot mask a failed build.
    mkdir -p "$target_dir/$target/release"
    printf 'failed:%s\n' "$target" > "$target_dir/$target/release/$binary"
    echo 'Finished (misleading stub output)'
    echo "unfiltered diagnostic: $tool $target" >&2
    exit 42
fi
if [[ "$target" != "${MISSING_TARGET:-}" ]]; then
    mkdir -p "$target_dir/$target/release"
    if [[ "$target" == "${EMPTY_TARGET:-}" ]]; then
        : > "$target_dir/$target/release/$binary"
    else
        printf '#!/bin/sh\n# fresh:%s\nexit 0\n' "$target" > "$target_dir/$target/release/$binary"
        chmod +x "$target_dir/$target/release/$binary"
    fi
fi
echo "unfiltered success: $tool $target"
STUB
    cp "$fixture/tools/cargo" "$fixture/tools/cross"
    chmod +x "$fixture/tools/cargo" "$fixture/tools/cross"
    FAIL_TARGET= MISSING_TARGET= EMPTY_TARGET=
}

run_build() {
    result=0
    PATH="$fixture/tools:$PATH" FIXTURE="$fixture" \
        SCRATCH="$scratch" TMPDIR="$scratch/../${scratch##*/}" \
        FAIL_TARGET="$FAIL_TARGET" MISSING_TARGET="$MISSING_TARGET" \
        EMPTY_TARGET="$EMPTY_TARGET" CARGO_TARGET_DIR="$fixture/foreign-target" \
        "$BASH" "$fixture/scripts/build-binaries.sh" > "$fixture/build.log" 2>&1 || result=$?
}

assert_unchanged() {
    for plugin in vscode intellij; do
        diff -r "$fixture/before-$plugin" "$fixture/$plugin/bin" || fail "$plugin changed on failure"
    done
}

assert_cleanup() {
    local leftovers
    leftovers="$(find "$scratch" -mindepth 1 -print)"
    [[ -z "$leftovers" ]] || fail "scratch artifacts remain: $leftovers"
    [[ ! -e "$fixture/tmp" ]] || fail "scratch directory created inside product"
    for spec in "${TARGETS[@]}"; do
        IFS=: read -r target platform binary <<< "$spec"
        [[ "$(< "$fixture/lsp/target/$target/release/$binary")" == "stale:$target" ]] || fail "existing target modified"
    done
}

assert_calls() {
    [[ "$(wc -l < "$fixture/calls")" -eq "$1" ]] || fail "unexpected build count"
}

without_bundles() {
    for plugin in vscode intellij; do
        mv "$fixture/$plugin/bin" "$fixture/unused-$plugin"
    done
}

assert_absent() {
    for plugin in vscode intellij; do
        [[ ! -e "$fixture/$plugin/bin" ]] || fail "$plugin bundle created on failure"
    done
}

for scenario in cargo-failure cross-failure missing-output empty-output; do
    setup "$scenario"
    expected=1
    calls=6
    case "$scenario" in
        cargo-failure) FAIL_TARGET=x86_64-pc-windows-msvc; expected=42; calls=1 ;;
        cross-failure) FAIL_TARGET=aarch64-apple-darwin; expected=42 ;;
        missing-output) MISSING_TARGET=aarch64-apple-darwin ;;
        empty-output) EMPTY_TARGET=aarch64-apple-darwin ;;
    esac
    run_build
    [[ "$result" -eq "$expected" ]] || fail "$scenario exit $result, expected $expected"
    assert_unchanged
    assert_calls "$calls"
    assert_cleanup
    if [[ -n "$FAIL_TARGET" ]]; then
        grep -Fq 'unfiltered diagnostic:' "$fixture/build.log" || fail "build diagnostic suppressed"
    fi
    echo "PASS: $scenario with stale source and existing bundles"
done

setup missing-no-bundles
without_bundles
MISSING_TARGET=aarch64-apple-darwin
run_build
[[ "$result" -eq 1 ]] || fail "missing output without old bundles succeeded"
assert_absent
assert_calls 6
assert_cleanup
echo 'PASS: missing output does not create partial bundles'

setup success
run_build
[[ "$result" -eq 0 ]] || fail "successful builds failed"
assert_calls 6
for plugin in vscode intellij; do
    [[ "$(find "$fixture/$plugin/bin" -type f | wc -l)" -eq 6 ]] || fail "wrong bundle size"
    for spec in "${TARGETS[@]}"; do
        IFS=: read -r target platform binary <<< "$spec"
        output="$fixture/$plugin/bin/$platform/$binary"
        [[ "$(< "$output")" == "$(printf '#!/bin/sh\n# fresh:%s\nexit 0' "$target")" ]] || fail "wrong $plugin binary for $target"
        if [[ "$binary" == ktav-lsp ]]; then
            [[ -x "$output" ]] || fail "executable mode lost: $output"
        fi
    done
done
grep -Fq 'unfiltered success:' "$fixture/build.log" || fail "nonmatching build output suppressed"
assert_cleanup
echo 'PASS: six successful locked builds replace both bundles'

setup staging-failure
cat > "$fixture/tools/cp" <<'STUB'
#!/bin/bash
set -euo pipefail
if [[ "${2:-}" == "$SCRATCH"/build-binaries.*/intellij/darwin-arm64/ktav-lsp ]]; then
    exit 44
fi
exec "$REAL_CP" "$@"
STUB
chmod +x "$fixture/tools/cp"
run_build
[[ "$result" -eq 44 ]] || fail "staging copy failure masked"
assert_unchanged
assert_calls 6
assert_cleanup
echo 'PASS: staging failure leaves both bundles unchanged'

for scenario in publication-failure publication-no-bundles old-bundle-move-failure; do
    setup "$scenario"
    if [[ "$scenario" == publication-no-bundles ]]; then
        without_bundles
    fi
    cat > "$fixture/tools/mv" <<'STUB'
#!/bin/bash
set -euo pipefail
if [[ "$1" == "$SCRATCH"/build-binaries.*/intellij && ! -f "$FIXTURE/move-failed" ]]; then
    : > "$FIXTURE/move-failed"
    mkdir -p "$2"
    printf 'partial publication\n' > "$2/partial"
    exit 43
fi
if [[ "$FIXTURE" == *'/old-bundle-move-failure fixture' &&
      "$2" == "$SCRATCH"/build-binaries.*/replaced-intellij ]]; then
    mkdir -p "$2"
    printf 'partial backup move\n' > "$2/partial"
    exit 45
fi
exec "$REAL_MV" "$@"
STUB
    chmod +x "$fixture/tools/mv"
    run_build
    expected=43
    if [[ "$scenario" == old-bundle-move-failure ]]; then
        expected=45
    fi
    [[ "$result" -eq "$expected" ]] || fail "publication failure masked"
    if [[ "$scenario" == publication-no-bundles ]]; then
        assert_absent
    else
        assert_unchanged
    fi
    assert_calls 6
    assert_cleanup
    echo "PASS: $scenario rolls back both bundles"
done
