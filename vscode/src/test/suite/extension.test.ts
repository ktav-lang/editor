import * as assert from "assert";
import * as path from "path";
import * as vscode from "vscode";

const FIXTURE_PATH = path.resolve(
  __dirname,
  "../../../../spec/versions/0.8/tests/invalid/bad_escape/escape_t_not_recognised.ktav",
);

function waitForDiagnostics(
  uri: vscode.Uri,
  predicate: (diagnostics: readonly vscode.Diagnostic[]) => boolean,
  timeoutMs: number,
): Promise<readonly vscode.Diagnostic[]> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      subscription.dispose();
      reject(new Error(`timed out waiting for diagnostics for ${uri.fsPath}`));
    }, timeoutMs);
    const subscription = vscode.languages.onDidChangeDiagnostics((event) => {
      if (!event.uris.some((changed) => changed.toString() === uri.toString())) {
        return;
      }
      const diagnostics = vscode.languages.getDiagnostics(uri);
      if (predicate(diagnostics)) {
        clearTimeout(timeout);
        subscription.dispose();
        resolve(diagnostics);
      }
    });
  });
}

suite("Ktav extension", () => {
  test("language `ktav` is registered", async () => {
    const langs = await vscode.languages.getLanguages();
    assert.ok(langs.includes("ktav"), "expected `ktav` in registered languages");
  });

  test("opens the 0.8 invalid fixture, activates, and reports its parse error", async function () {
    this.timeout(30_000);
    const serverPath = process.env.KTAV_LSP_PATH;
    assert.ok(serverPath, "KTAV_LSP_PATH must point to the ktav-lsp binary");

    await vscode.workspace
      .getConfiguration("ktav")
      .update(
        "server.path",
        serverPath,
        vscode.ConfigurationTarget.Global,
      );

    const uri = vscode.Uri.file(FIXTURE_PATH);
    const diagnosticsPromise = waitForDiagnostics(
      uri,
      (diagnostics) => diagnostics.length > 0,
      20_000,
    );
    const doc = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(doc);

    assert.strictEqual(doc.languageId, "ktav");
    const extension = vscode.extensions.getExtension("ktav-lang.ktav");
    assert.ok(extension, "extension `ktav-lang.ktav` should be installed");

    const diagnostics = await diagnosticsPromise;
    assert.ok(extension.isActive, "opening a Ktav document should activate the extension");
    assert.strictEqual(diagnostics.length, 1);
    assert.strictEqual(diagnostics[0].source, "ktav");
    assert.strictEqual(diagnostics[0].severity, vscode.DiagnosticSeverity.Error);
    assert.match(diagnostics[0].message, /BadEscapeSequence/);
  });
});
