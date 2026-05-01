import * as assert from "assert";
import * as os from "os";
import * as path from "path";
import * as fs from "fs";
import * as cp from "child_process";
import * as vscode from "vscode";
import { State } from "vscode-languageclient/node";

function ktavLspOnPath(): boolean {
  const which = process.platform === "win32" ? "where" : "which";
  try {
    cp.execFileSync(which, ["ktav-lsp"], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

suite("Ktav extension", () => {
  test("language `ktav` is registered", async () => {
    const langs = await vscode.languages.getLanguages();
    assert.ok(langs.includes("ktav"), "expected `ktav` in registered languages");
  });

  test("opening a .ktav file activates the extension and the LSP reaches Running", async function () {
    if (!process.env.KTAV_LSP_PATH && !ktavLspOnPath()) {
      // No way to reach a server — skip rather than green-light a no-op.
      this.skip();
      return;
    }
    this.timeout(30_000);

    if (process.env.KTAV_LSP_PATH) {
      await vscode.workspace
        .getConfiguration("ktav")
        .update(
          "server.path",
          process.env.KTAV_LSP_PATH,
          vscode.ConfigurationTarget.Global,
        );
    }

    const tmp = path.join(os.tmpdir(), `ktav-activation-${Date.now()}.ktav`);
    fs.writeFileSync(tmp, "name: hello\nport: $i 8080\n", "utf8");

    const doc = await vscode.workspace.openTextDocument(tmp);
    await vscode.window.showTextDocument(doc);

    assert.strictEqual(doc.languageId, "ktav");

    const ext = vscode.extensions.getExtension("ktav-lang.ktav");
    assert.ok(ext, "extension `ktav-lang.ktav` should be installed");
    await ext!.activate();
    assert.ok(ext!.isActive, "extension should be active after activate() resolved");

    // Probe the language client state via the running extension's exports
    // is brittle; instead, watch publishDiagnostics — that proves the LSP
    // actually connected and processed the file.
    const deadline = Date.now() + 20_000;
    let sawDiagsEvent = false;
    const sub = vscode.languages.onDidChangeDiagnostics((e) => {
      if (e.uris.some((u) => u.toString() === doc.uri.toString())) {
        sawDiagsEvent = true;
      }
    });
    try {
      while (Date.now() < deadline && !sawDiagsEvent) {
        await new Promise((r) => setTimeout(r, 250));
      }
    } finally {
      sub.dispose();
    }
    assert.ok(sawDiagsEvent, "expected at least one publishDiagnostics event for the open .ktav file");
  });

  test("diagnostics arrive when KTAV_LSP_PATH points at a real server", async function () {
    if (!process.env.KTAV_LSP_PATH) {
      this.skip();
      return;
    }
    this.timeout(60_000);

    await vscode.workspace
      .getConfiguration("ktav")
      .update(
        "server.path",
        process.env.KTAV_LSP_PATH,
        vscode.ConfigurationTarget.Global,
      );

    const tmp = path.join(os.tmpdir(), `ktav-bad-${Date.now()}.ktav`);
    fs.writeFileSync(tmp, "key: { unclosed\n", "utf8");
    const doc = await vscode.workspace.openTextDocument(tmp);
    await vscode.window.showTextDocument(doc);

    const deadline = Date.now() + 10_000;
    let diags: readonly vscode.Diagnostic[] = [];
    while (Date.now() < deadline) {
      diags = vscode.languages.getDiagnostics(doc.uri);
      if (diags.length > 0) break;
      await new Promise((r) => setTimeout(r, 250));
    }
    assert.ok(diags.length > 0, "expected at least one diagnostic from ktav-lsp");
  });
});

// Keep `State` referenced so the import isn't pruned by tsc's unused-import
// check — it's part of the public diagnostic vocabulary even though we
// observe state via diagnostics events above.
void State;
