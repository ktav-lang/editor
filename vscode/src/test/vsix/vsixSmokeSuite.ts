import * as assert from "assert";
import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";

function waitForDiagnostic(uri: vscode.Uri, timeoutMs: number): Promise<readonly vscode.Diagnostic[]> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      subscription.dispose();
      reject(new Error(`timed out waiting for Ktav diagnostic: ${uri.fsPath}`));
    }, timeoutMs);
    const subscription = vscode.languages.onDidChangeDiagnostics((event) => {
      if (!event.uris.some((changed) => changed.toString() === uri.toString())) return;
      const diagnostics = vscode.languages.getDiagnostics(uri);
      if (diagnostics.length === 0) return;
      clearTimeout(timeout);
      subscription.dispose();
      resolve(diagnostics);
    });
    const existing = vscode.languages.getDiagnostics(uri);
    if (existing.length > 0) {
      clearTimeout(timeout);
      subscription.dispose();
      resolve(existing);
    }
  });
}

export async function run(): Promise<void> {
  const expectedVersion = process.env.KTAV_VSIX_EXPECTED_VERSION;
  const extensionsDir = process.env.KTAV_VSIX_EXTENSIONS_DIR;
  const fixture = process.env.KTAV_VSIX_FIXTURE;
  assert.ok(expectedVersion && extensionsDir && fixture, "smoke inputs missing");
  assert.strictEqual(process.env.KTAV_LSP_PATH, undefined);
  assert.strictEqual(process.env.NODE_PATH, undefined);
  const extension = vscode.extensions.getExtension("ktav-lang.ktav");
  assert.ok(extension, "installed ktav-lang.ktav not found");
  assert.strictEqual(extension.packageJSON.version, expectedVersion);
  const installedRoot = fs.realpathSync(extension.extensionPath);
  const isolatedRoot = fs.realpathSync(extensionsDir);
  assert.ok(installedRoot.startsWith(isolatedRoot + path.sep), `not installed from VSIX: ${installedRoot}`);
  const config = vscode.workspace.getConfiguration("ktav").inspect<string>("server.path");
  assert.strictEqual(config?.globalValue, undefined);
  assert.strictEqual(config?.workspaceValue, undefined);
  assert.strictEqual(config?.workspaceFolderValue, undefined);
  assert.strictEqual(config?.defaultValue, "");
  const binary = path.join(installedRoot, "bin", "linux-x64", "ktav-lsp");
  fs.accessSync(binary, fs.constants.X_OK);
  for (const directory of (process.env.PATH ?? "").split(path.delimiter)) {
    assert.ok(!fs.existsSync(path.join(directory, "ktav-lsp")), "ambient ktav-lsp would mask failed discovery");
  }

  const uri = vscode.Uri.file(fixture);
  const diagnostic = waitForDiagnostic(uri, 25_000);
  const document = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(document);
  assert.strictEqual(document.languageId, "ktav");
  const received = await diagnostic;
  assert.ok(extension.isActive, "installed extension did not activate");
  assert.strictEqual(received.length, 1);
  assert.strictEqual(received[0].source, "ktav");
  assert.strictEqual(received[0].severity, vscode.DiagnosticSeverity.Error);
  assert.match(received[0].message, /\bBadEscapeSequence\b/);
}
