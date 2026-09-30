import * as assert from "assert";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { promisify } from "util";
import { execFile } from "child_process";
import {
  downloadAndUnzipVSCode,
  resolveCliPathFromVSCodeExecutablePath,
  runTests,
} from "@vscode/test-electron";

async function main(): Promise<void> {
  const vsix = process.argv[2];
  assert.ok(vsix && path.isAbsolute(vsix), "pass absolute validated VSIX path");
  const root = path.resolve(__dirname, "..", "..", "..");
  const manifest = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
  const fixture = path.resolve(
    root, "..", "spec", "versions", "0.8", "tests", "invalid",
    "bad_escape", "escape_t_not_recognised.ktav",
  );
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "ktav-vsix-smoke-"));
  const extensionsDir = path.join(temp, "extensions");
  const userDataDir = path.join(temp, "user-data");
  const fixtureCopy = path.join(temp, "escape_t_not_recognised.ktav");
  fs.mkdirSync(extensionsDir);
  fs.mkdirSync(userDataDir);
  fs.copyFileSync(fixture, fixtureCopy);

  try {
    const executable = await downloadAndUnzipVSCode();
    const cli = resolveCliPathFromVSCodeExecutablePath(executable);
    const runCli = promisify(execFile);
    await runCli(cli, [
      "--install-extension", vsix,
      `--extensions-dir=${extensionsDir}`,
      `--user-data-dir=${userDataDir}`,
      "--force",
    ], { timeout: 60_000 });
    const env = { ...process.env };
    delete env.KTAV_LSP_PATH;
    delete env.NODE_PATH;
    await runTests({
      vscodeExecutablePath: executable,
      extensionDevelopmentPath: path.join(root, "src", "test", "vsix", "driver"),
      extensionTestsPath: path.join(__dirname, "vsixSmokeSuite.js"),
      extensionTestsEnv: {
        ...env,
        KTAV_LSP_PATH: undefined,
        NODE_PATH: undefined,
        KTAV_VSIX_EXPECTED_VERSION: manifest.version,
        KTAV_VSIX_EXTENSIONS_DIR: extensionsDir,
        KTAV_VSIX_FIXTURE: fixtureCopy,
      },
      launchArgs: [
        fixtureCopy,
        `--extensions-dir=${extensionsDir}`,
        `--user-data-dir=${userDataDir}`,
      ],
    });
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
}

void main().catch((error) => {
  console.error("VSIX smoke failed:", error);
  process.exitCode = 1;
});
