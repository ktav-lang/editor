// Pure server-discovery logic. Deliberately free of `vscode` imports so
// it can be unit-tested under plain Mocha (no @vscode/test-electron, no
// Electron host). All side-effecting dependencies (config getter,
// fs.existsSync, platform/arch, extensionPath) are injected.

import * as path from "path";

export const BINARY_NAME = "ktav-lsp";

export type NodePlatform = NodeJS.Platform;
export type NodeArch = NodeJS.Architecture;

/** Compute the bundled-binary subdir name (e.g. `linux-x64`, `win32-arm64`). */
export function platformDir(platform: NodePlatform, arch: NodeArch): string {
  return `${platform}-${arch}`;
}

/** Append `.exe` on Windows, nothing elsewhere. */
export function exeName(platform: NodePlatform): string {
  return platform === "win32" ? `${BINARY_NAME}.exe` : BINARY_NAME;
}

export interface DiscoveryDeps {
  /** Returns the value of the `ktav.server.path` setting (raw, possibly empty). */
  getServerPathSetting: () => string;
  /** `fs.existsSync` — injected so tests can stub a virtual filesystem. */
  existsSync: (p: string) => boolean;
  platform: NodePlatform;
  arch: NodeArch;
  extensionPath: string;
}

/**
 * Discovery order (mirrors the IntelliJ plugin):
 *   1. explicit `ktav.server.path` setting (trimmed; empty / non-existent → skip)
 *   2. bundled binary at `<extensionPath>/bin/<platform>-<arch>/ktav-lsp[.exe]`
 *   3. bare `ktav-lsp` — let the OS resolve it via PATH
 */
export function resolveServerPath(deps: DiscoveryDeps): string {
  const explicit = (deps.getServerPathSetting() ?? "").trim();
  if (explicit.length > 0 && deps.existsSync(explicit)) {
    return explicit;
  }

  const bundled = path.join(
    deps.extensionPath,
    "bin",
    platformDir(deps.platform, deps.arch),
    exeName(deps.platform),
  );
  if (deps.existsSync(bundled)) {
    return bundled;
  }

  return BINARY_NAME;
}
