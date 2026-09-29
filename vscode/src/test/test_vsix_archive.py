import json
import tempfile
import unittest
import zipfile
from pathlib import Path

from vsix_archive import BINARIES, RUNTIME, validate


def sample() -> dict[str, tuple[bytes, int]]:
    manifest = {
        "publisher": "ktav-lang", "name": "ktav", "version": "0.8.0",
        "main": "./out/extension.js",
        "contributes": {
            "languages": [{"id": "ktav", "configuration": "./syntaxes/language-configuration.json"}],
            "grammars": [{"language": "ktav", "path": "./syntaxes/ktav.tmLanguage.json"}],
            "configuration": {"properties": {"ktav.server.path": {"default": ""}}},
        },
    }
    files = {
        "package.json": (json.dumps(manifest).encode(), 0o644),
        "out/extension.js": (b'require("./discovery");require("vscode-languageclient/node")', 0o644),
        "out/discovery.js": (b'bin/linux-x64/ktav-lsp', 0o644),
        "syntaxes/ktav.tmLanguage.json": (b'{"scopeName":"source.ktav","patterns":[{}]}', 0o644),
        "syntaxes/language-configuration.json": (b'{"brackets":[["{","}"]],"comments":{"lineComment":"##"}}', 0o644),
    }
    files.update({name: (b"runtime", 0o644) for name in RUNTIME})
    files.update({f"bin/{name}": (b"binary", 0o755) for name in BINARIES})
    return files


class ArchiveValidationTest(unittest.TestCase):
    def check(self, files: dict[str, tuple[bytes, int]], valid: bool) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "test.vsix"
            with zipfile.ZipFile(archive, "w") as output:
                for name, (data, mode) in files.items():
                    info = zipfile.ZipInfo(f"extension/{name}")
                    info.create_system = 3
                    info.external_attr = (0o100000 | mode) << 16
                    output.writestr(info, data)
            if valid:
                validate(archive, "0.8.0")
            else:
                with self.assertRaises(ValueError):
                    validate(archive, "0.8.0")

    def test_complete_archive(self) -> None:
        self.check(sample(), True)

    def test_missing_runtime_dependency(self) -> None:
        files = sample()
        del files[RUNTIME[0]]
        self.check(files, False)

    def test_missing_platform_binary(self) -> None:
        files = sample()
        del files[f"bin/{BINARIES[-1]}"]
        self.check(files, False)

    def test_empty_platform_binary(self) -> None:
        files = sample()
        files[f"bin/{BINARIES[0]}"] = (b"", 0o755)
        self.check(files, False)

    def test_missing_execute_bit(self) -> None:
        files = sample()
        files[f"bin/{BINARIES[0]}"] = (b"binary", 0o644)
        self.check(files, False)

    def test_wrong_version(self) -> None:
        files = sample()
        manifest = json.loads(files["package.json"][0])
        manifest["version"] = "0.0.0"
        files["package.json"] = (json.dumps(manifest).encode(), 0o644)
        self.check(files, False)

    def test_missing_discovery(self) -> None:
        files = sample()
        del files["out/discovery.js"]
        self.check(files, False)

    def test_test_code_is_excluded(self) -> None:
        files = sample()
        files["out/test/runVsixSmoke.js"] = (b"test", 0o644)
        self.check(files, False)


if __name__ == "__main__":
    unittest.main()
