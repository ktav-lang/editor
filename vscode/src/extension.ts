import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext): void {
  console.log("Ktav language extension activated");
  // Future: spawn ktav-lsp and register a LanguageClient.
}

export function deactivate(): void {}
