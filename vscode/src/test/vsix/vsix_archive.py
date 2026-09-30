"""Fail closed on release-critical VSIX contents before any publication."""

import json
import sys
import zipfile
from pathlib import Path


BINARIES = (
    "linux-x64/ktav-lsp",
    "linux-arm64/ktav-lsp",
    "darwin-x64/ktav-lsp",
    "darwin-arm64/ktav-lsp",
    "win32-x64/ktav-lsp.exe",
    "win32-arm64/ktav-lsp.exe",
)
RUNTIME = (
    "node_modules/vscode-languageclient/node.js",
    "node_modules/vscode-languageclient/lib/node/main.js",
    "node_modules/vscode-languageserver-protocol/node.js",
    "node_modules/vscode-languageserver-protocol/lib/node/main.js",
)


def validate(archive: Path, expected_version: str) -> None:
    with zipfile.ZipFile(archive) as vsix:
        entries = vsix.infolist()
        files = {item.filename: item for item in entries}
        if len(files) != len(entries):
            raise ValueError("duplicate ZIP entries")
        if any(name.startswith("extension/out/test/") for name in files):
            raise ValueError("test code included in VSIX")
        bad = vsix.testzip()
        if bad:
            raise ValueError(f"corrupt ZIP entry: {bad}")

        def read(name: str) -> bytes:
            entry = f"extension/{name}"
            if entry not in files or files[entry].file_size == 0:
                raise ValueError(f"missing or empty VSIX entry: {entry}")
            return vsix.read(entry)

        manifest = json.loads(read("package.json"))
        if (manifest.get("publisher"), manifest.get("name"), manifest.get("version")) != (
            "ktav-lang", "ktav", expected_version
        ):
            raise ValueError("wrong extension identity or version")
        if manifest.get("main") != "./out/extension.js":
            raise ValueError("wrong extension entry point")
        contributes = manifest.get("contributes", {})
        languages = contributes.get("languages", [])
        grammars = contributes.get("grammars", [])
        if not any(lang.get("id") == "ktav" and lang.get("configuration") == "./syntaxes/language-configuration.json" for lang in languages):
            raise ValueError("Ktav language configuration not registered")
        if not any(grammar.get("language") == "ktav" and grammar.get("path") == "./syntaxes/ktav.tmLanguage.json" for grammar in grammars):
            raise ValueError("Ktav grammar not registered")
        if manifest.get("contributes", {}).get("configuration", {}).get("properties", {}).get("ktav.server.path", {}).get("default") != "":
            raise ValueError("server path must default to auto-discovery")

        extension = read("out/extension.js")
        discovery = read("out/discovery.js")
        if b"./discovery" not in extension or b"vscode-languageclient/node" not in extension:
            raise ValueError("extension entry point lacks discovery or language client")
        if b"bin" not in discovery or b"ktav-lsp" not in discovery:
            raise ValueError("bundled server discovery missing")
        for name in RUNTIME:
            read(name)
        grammar = json.loads(read("syntaxes/ktav.tmLanguage.json"))
        config = json.loads(read("syntaxes/language-configuration.json"))
        if grammar.get("scopeName") != "source.ktav" or not grammar.get("patterns"):
            raise ValueError("Ktav grammar is not usable")
        if not config.get("brackets") or not config.get("comments"):
            raise ValueError("Ktav language configuration is not usable")
        for name in BINARIES:
            read(f"bin/{name}")
            info = files[f"extension/bin/{name}"]
            if not name.endswith(".exe") and not (info.external_attr >> 16) & 0o111:
                raise ValueError(f"binary has no POSIX execute bit: {name}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: vsix_archive.py <archive.vsix> <expected-version>")
    try:
        validate(Path(sys.argv[1]), sys.argv[2])
    except (ValueError, KeyError, zipfile.BadZipFile) as exc:
        raise SystemExit(f"VSIX validation failed: {exc}") from exc
    print("VSIX archive contents and modes validated")
