// Pure unit tests for src/discovery.ts. Runs under plain Mocha — no
// vscode module, no Electron host.

import * as assert from "assert";
import * as path from "path";
import {
  BINARY_NAME,
  exeName,
  platformDir,
  resolveServerPath,
} from "../../discovery";

function existsOnly(allowed: Set<string>): (p: string) => boolean {
  return (p: string) => allowed.has(p);
}

suite("discovery: platformDir / exeName", () => {
  test("linux x64", () => assert.strictEqual(platformDir("linux", "x64"), "linux-x64"));
  test("linux arm64", () =>
    assert.strictEqual(platformDir("linux", "arm64"), "linux-arm64"));
  test("darwin x64", () =>
    assert.strictEqual(platformDir("darwin", "x64"), "darwin-x64"));
  test("darwin arm64", () =>
    assert.strictEqual(platformDir("darwin", "arm64"), "darwin-arm64"));
  test("win32 x64", () =>
    assert.strictEqual(platformDir("win32", "x64"), "win32-x64"));

  test("windows binary has .exe suffix", () =>
    assert.strictEqual(exeName("win32"), "ktav-lsp.exe"));
  test("non-windows binary has no suffix", () => {
    assert.strictEqual(exeName("linux"), "ktav-lsp");
    assert.strictEqual(exeName("darwin"), "ktav-lsp");
  });
});

suite("discovery: resolveServerPath", () => {
  const extPath = "/ext";

  test("setting points to existing absolute path → returned", () => {
    const explicit = "/opt/bin/ktav-lsp";
    const got = resolveServerPath({
      getServerPathSetting: () => explicit,
      existsSync: existsOnly(new Set([explicit])),
      platform: "linux",
      arch: "x64",
      extensionPath: extPath,
    });
    assert.strictEqual(got, explicit);
  });

  test("setting empty + bundled binary at bin/linux-x64/ktav-lsp exists → returned", () => {
    const bundled = path.join(extPath, "bin", "linux-x64", "ktav-lsp");
    const got = resolveServerPath({
      getServerPathSetting: () => "",
      existsSync: existsOnly(new Set([bundled])),
      platform: "linux",
      arch: "x64",
      extensionPath: extPath,
    });
    assert.strictEqual(got, bundled);
  });

  test("setting empty + no bundled → falls through to bare ktav-lsp", () => {
    const got = resolveServerPath({
      getServerPathSetting: () => "",
      existsSync: () => false,
      platform: "linux",
      arch: "x64",
      extensionPath: extPath,
    });
    assert.strictEqual(got, BINARY_NAME);
  });

  test("setting points to non-existent path → falls through", () => {
    const got = resolveServerPath({
      getServerPathSetting: () => "/no/such/path",
      existsSync: () => false,
      platform: "linux",
      arch: "x64",
      extensionPath: extPath,
    });
    assert.strictEqual(got, BINARY_NAME);
  });

  test("whitespace-only setting is treated as empty", () => {
    const bundled = path.join(extPath, "bin", "darwin-arm64", "ktav-lsp");
    const got = resolveServerPath({
      getServerPathSetting: () => "   \t",
      existsSync: existsOnly(new Set([bundled])),
      platform: "darwin",
      arch: "arm64",
      extensionPath: extPath,
    });
    assert.strictEqual(got, bundled);
  });

  test("windows bundled binary uses .exe suffix", () => {
    const bundled = path.join(extPath, "bin", "win32-x64", "ktav-lsp.exe");
    const got = resolveServerPath({
      getServerPathSetting: () => "",
      existsSync: existsOnly(new Set([bundled])),
      platform: "win32",
      arch: "x64",
      extensionPath: extPath,
    });
    assert.strictEqual(got, bundled);
  });

  test("non-windows bundled binary has no .exe suffix", () => {
    // existsSync returns true for the suffixless path only — so if the code
    // wrongly looked for `.exe`, the bundled branch would miss and we'd
    // fall through to BINARY_NAME.
    const bundled = path.join(extPath, "bin", "linux-arm64", "ktav-lsp");
    const got = resolveServerPath({
      getServerPathSetting: () => "",
      existsSync: existsOnly(new Set([bundled])),
      platform: "linux",
      arch: "arm64",
      extensionPath: extPath,
    });
    assert.strictEqual(got, bundled);
  });

  test("explicit setting trimmed before existence check", () => {
    const real = "/opt/bin/ktav-lsp";
    const got = resolveServerPath({
      getServerPathSetting: () => `  ${real}\t`,
      existsSync: existsOnly(new Set([real])),
      platform: "linux",
      arch: "x64",
      extensionPath: extPath,
    });
    assert.strictEqual(got, real);
  });
});
